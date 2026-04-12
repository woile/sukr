; Highlights for Just

[
  "export"
  "import"
  "alias"
  "set"
  "if"
  "else"
] @keyword

(recipe_header name: (identifier) @function)

(parameter name: (identifier) @variable.parameter)

(assignment left: (identifier) @variable)

(attribute (identifier) @attribute)
(attribute_kv_argument key: (identifier) @variable.parameter)

(comment) @comment
(shebang) @comment

[
  (string)
  (external_command)
] @string
