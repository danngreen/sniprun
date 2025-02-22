use crate::*;
use error::SniprunError;
use interpreter::{Interpreter, SupportLevel};
use std::io::prelude::*;
use std::process::Command;
use std::{fs::File, io::Read};

pub struct Launcher {
    pub data: DataHolder,
}

impl Launcher {
    pub fn new(data: DataHolder) -> Self {
        Launcher { data }
    }

    pub fn select_and_run<'a>(&self) -> Result<String, SniprunError> {
        let selection = self.select();
        if let Some((name, level)) = selection {
            //launch !
            iter_types! {
                if Current::get_name() == name {
                    info!("[LAUNCHER] Selected interpreter: {}, at level {}", name, level);
                    let mut inter = Current::new_with_level(self.data.clone(), level);
                    return inter.run();
                }
            }
            info!("[LAUNCHER] Could not find a suitable interpreter");
            return Err(SniprunError::CustomError(
                "could not find/run the selected interpreter".to_owned(),
            ));
        } else {
            return Err(SniprunError::CustomError(String::from(
                "No filetype set for current file",
            )));
        }
    }

    pub fn select(&self) -> Option<(String, SupportLevel)> {
        if self.data.filetype.is_empty() {
            return None;
        }

        let mut max_level_support = SupportLevel::Unsupported;
        let mut name_best_interpreter = String::from("Generic");
        //select the best interpreter for the language
        let mut skip_all = false;
        iter_types! {
            if !skip_all && Current::get_supported_languages().contains(&self.data.filetype){
                if Current::get_max_support_level() > max_level_support {
                    max_level_support = Current::get_max_support_level();
                    name_best_interpreter = Current::get_name();
                }

                if self.data.selected_interpreters.contains(&Current::get_name()){
                    max_level_support = SupportLevel::Selected;
                    name_best_interpreter = Current::get_name();
                    skip_all = true;
                }

                if Current::default_for_filetype() {
                    max_level_support = Current::get_max_support_level();
                    name_best_interpreter = Current::get_name();
                    skip_all = true;
                }
            }
        }
        let _ = skip_all; //silence false unused variable warning
        return Some((name_best_interpreter, max_level_support));
    }

    pub fn info(&self) -> std::io::Result<String> {
        let mut v: Vec<String> = vec![];
        let filename = self.data.sniprun_root_dir.clone() + "/ressources/asciiart.txt";

        if let Ok(mut file) = File::open(filename) {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            info!("[INFO] Retrieved asciiart");
            v.push(content);
            v.push("\n".to_owned());
        } else {
            v.push(String::from("SNIPRUN\n"));
        }

        let gitscript = self.data.sniprun_root_dir.clone() + "/ressources/gitscript.sh";
        let mut get_version = Command::new(gitscript);
        get_version.current_dir(self.data.sniprun_root_dir.clone());
        if let Ok(res) = get_version.output(){
            info!("gitscript result: {:?}", res);
            if res.status.success() {
                let online_version = String::from_utf8(res.stdout).unwrap();
                info!("online version available: {}", &online_version);
                v.push(online_version);
            } else {
                v.push(String::from("Could not determine up-to-date status\n"));
            }
        } else {
            v.push(String::from("Could not determine up-to-date status\n"));
        }

        if let Some((name, level)) = self.select() {
            v.push(format!(
                "\nCurrently selected interpreter: {}, at support level: {}\n",
                name, level
            ));
        } else {
            v.push("No interpreter selected\n".to_string());
        }

        let separator = "|--------------------------|--------------|---------------|-------------|------------|--------------|------------|".to_string();
        v.push(separator.clone());
        v.push("| Interpreter              | Language     | Support Level | Default for |    REPL    | REPL enabled | Treesitter |".to_string());
        v.push("|                          |              |               |  filetype   | capability |  by default  | capability |".to_string());

        let mut temp_vec = vec![];
        iter_types! {
            let line = format!("| {:<25}| {:<13}| {:<14}|{:^13}|{:^12}|{:^14}|{:^12}|",
                    Current::get_name(),
                    Current::get_supported_languages().iter().next().unwrap_or(&"".to_string()),
                    Current::get_max_support_level().to_string(),
                    match Current::default_for_filetype() {true => "yes" ,false => "no"},
                    match Current::has_repl_capability() { true => "yes" ,false => "no"},
                    match Current::behave_repl_like_default() { true => "yes" ,false => "no"},
                    match Current::has_treesitter_capability() { true => "yes" ,false => "no"}
                    ).to_string();
            temp_vec.push(line);
        }

        temp_vec.sort();

        for (i, line) in temp_vec.iter().enumerate() {
            if i % 3 == 0 {
                v.push(separator.clone());
            }
            v.push(line.to_string());
        }

        v.push(separator.clone());

        if self.data.return_message_type == ReturnMessageType::Multiline {
            info!("[INFO] Returning info directly");
            return Ok(v.join("\n"));
        } else {
            //write to infofile
            info!("[INFO] Writing info to file");
            let filename = self.data.sniprun_root_dir.clone() + "/ressources/infofile.txt";
            let mut file = File::create(filename).unwrap();
            file.write_all(v.join("\n").as_bytes()).unwrap();
            return Ok("".to_owned());
        }
    }
}

#[cfg(test)]
mod test_launcher {

    use super::*;
    use std::env;

    #[test]
    fn run() {
        let mut data = DataHolder::new();
        data.filetype = String::from("pyt");
        data.current_line = String::from("println!(\"Hello\");");
        data.current_bloc = String::from("println!(\"Hello\");");
        data.range = [1, 1];

        let launcher = Launcher::new(data);
        let _res = launcher.select();
    }

    #[test]
    fn info() {
        let mut data = DataHolder::new();
        let path = env::current_dir().unwrap();
        data.sniprun_root_dir = path.display().to_string();

        data.filetype = String::from("rust");
        data.current_line = String::from("println!(\"Hello\");");
        data.current_bloc = String::from("println!(\"Hello\");");
        data.range = [1, 1];

        let launcher = Launcher::new(data);
        let _res = launcher.info().unwrap();
    }
}
