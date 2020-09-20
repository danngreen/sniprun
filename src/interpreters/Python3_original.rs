//Interpreter:| Python3_original    | python3     |
//############|_____________________|_____________|________________<- delimiters to help formatting,
//###########| Interpretername      | language    | comment
// Keep (but modify the first line after the :) if you wish to have this interpreter listedvia SnipList
#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Python3_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    imports: String,
}

fn module_used(line: &str, code: &str) -> bool {
    if line.contains("*") {
        return true;
    }
    if line.contains(" as ") {
        if let Some(name) = line.split(" ").last() {
            return code.contains(name);
        }
    }
    for name in line
        .replace(",", " ")
        .replace("from", " ")
        .replace("import ", " ")
        .split(" ")
    {
        if code.contains(name.trim()) {
            return true;
        }
    }
    return false;
}

impl Python3_original {
    pub fn fetch_imports(&mut self) -> std::io::Result<()> {
        if self.support_level < SupportLevel::Line {
            return Ok(());
        }
        //no matter if it fails, we should try to run the rest
        let mut file = File::open(&self.data.filepath)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        for line in contents.lines() {
            // info!("lines are : {}", line);
            if line.contains("import ") //basic selection
                && line.trim().chars().next() != Some('#')
            && module_used(line, &contents)
            {
                // embed in try catch blocs in case uneeded module is unavailable
                self.imports = self.imports.clone()
                    + "\n
try:\n" + "\t" + line
                    + "\nexcept:\n\t"
                    + "print()\n";
            }
        }
        Ok(())
    }
}

impl Interpreter for Python3_original {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Python3_original> {
        Box::new(Python3_original {
            data,
            support_level: level,
            code: String::from(""),
            imports: String::from(""),
        })
    }

    fn get_name() -> String {
        String::from("Python3_original")
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("python"),
            String::from("python3"),
            String::from("py"),
        ]
    }

    fn get_current_level(&self) -> SupportLevel {
        self.support_level
    }
    fn set_current_level(&mut self, level: SupportLevel) {
        self.support_level = level;
    }

    fn get_data(&self) -> DataHolder {
        self.data.clone()
    }

    fn get_max_support_level() -> SupportLevel {
        SupportLevel::File
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        let _res = self.fetch_imports();
        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
            && self.get_current_level() >= SupportLevel::Bloc
        {
            self.code = self.data.current_bloc.clone();
        } else if !self.data.current_line.replace(" ", "").is_empty()
            && self.get_current_level() >= SupportLevel::Line
        {
            self.code = self.data.current_line.clone();
        } else {
            self.code = String::from("");
        }

        if self.get_current_level() >= SupportLevel::File {
            let mut code_to_add = String::new();
            let ranges = self.get_code_dependencies().unwrap_or(vec![]);
            let mut file = File::open(&self.data.filepath).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            for range in ranges {
                for (i, line) in contents.lines().enumerate() {
                    if i < range.start_row {
                        continue;
                    } else if i == range.start_row {
                        code_to_add.push_str(&line[range.start_col..]);
                    } else if i == range.end_row {
                        code_to_add.push_str(&line[..range.end_col]);
                    } else if i > range.end_row {
                        continue;
                    } else {
                        //is in the middle of the range
                        code_to_add.push_str(&line);
                    }
                }
            }
            self.code = code_to_add + &self.code;
        }
        Ok(())
    }
    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        self.code = self.imports.clone()
            + &String::from(
                "from io import StringIO
import sys

sys.stdout = mystdout1427851999 = StringIO()

",
            )
            + &unindent(&format!("{}{}", "\n", self.code.as_str()))
            + "
exit_value1428571999 = str(mystdout1427851999.getvalue())";
        Ok(())
    }
    fn build(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }
    fn execute(&mut self) -> Result<String, SniprunError> {
        let py = pyo3::Python::acquire_gil();
        let locals = PyDict::new(py.python());
        match py.python().run(self.code.as_str(), None, Some(locals)) {
            Ok(_) => (),
            Err(_e) => {
                return Err(SniprunError::InterpreterError);
            }
        }
        let py_stdout = locals.get_item("exit_value1428571999");
        if let Some(unwrapped_stdout) = py_stdout {
            let result: Result<String, _> = unwrapped_stdout.extract();
            match result {
                Ok(unwrapped_result) => return Ok(unwrapped_result),
                Err(_e) => return Err(SniprunError::InterpreterError),
            }
        } else {
            return Err(SniprunError::InterpreterLimitationError(String::from(
                "Code erased a needed value to get standart output)",
            )));
        }
    }
}
