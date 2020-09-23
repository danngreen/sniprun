local locals = require'nvim-treesitter.locals'
local parsers = require'nvim-treesitter.parsers'
local ts_utils = require'nvim-treesitter.ts_utils'
local utils = require'nvim-treesitter.utils'
local query_module = require'nvim-treesitter.query'
local api=vim.api

local M = {}

function M.get_definition_scope_of_function_node(node, bufnr)
  -- local node = ts_utils.get_node_at_cursor()
  local bufnr = bufnr or api.nvim_get_current_buf()

  local definition = locals.find_definition(node, bufnr)


  local containing_scope = locals.containing_scope(definition, bufnr)

  return ts_utils.get_node_range(containing_scope)
end



-- ! the end row is exclusive so you'll often need to add 1
function M.list_nodes_in_range(start_row, end_row, bufnr)
  local bufnr = bufnr or api.nvim_get_current_buf()

  -- get a table of all function call in the scope
    -- get a table of all method calls
    -- get a table of all used variables
    --
    --
  local query_group = 'function_defs'
  local lang = parsers.get_buf_lang(bufnr)
  print("lang= ",lang)
  if not lang then return function() end end

  local query = query_module.get_query(lang, query_group)
  if not query then return function() end end
  print("query= ",query)

  local parser = parsers.get_parser(bufnr, lang)
  if not parser then return function() end end
  print("parser= ",parser)

  local root = parser:parse():root()

  print("root= ",root)


  local function_iterator = M.iter_prepared_matches(query, root, bufnr, start_row, end_row)
  --cheating...
--  fnode = ts_utils.get_node_at_cursor()
--  function_calls = {}
--  table.insert(function_calls, fnode)


 for q in function_iterator do
   print(q)
 end

--  for call do
--    print(M.get_definition_scope_of_function_node(call, bufnr))
--    end

end





return M

