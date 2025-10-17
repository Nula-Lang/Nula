// internal/ast/ast.go - AST definitions

package ast

type Node interface{}

type Program struct {
	Statements []Node
}

type VarDecl struct {
	Name  string
	Value Node
}

type Assign struct {
	Name  string
	Value Node
}

type IfStmt struct {
	Condition Node
	Then      []Node
	Else      []Node
}

type WhileStmt struct {
	Condition Node
	Body      []Node
}

type ForStmt struct {
	Var   string
	Start Node
	End   Node
	Body  []Node
}

type FuncDef struct {
	Name   string
	Params []string
	Body   []Node
}

type FuncCall struct {
	Name string
	Args []Node
}

type BinOp struct {
	Op    string
	Left  Node
	Right Node
}

type Literal struct {
	Value float64
}

type StrLit struct {
	Value string
}

type Var struct {
	Name string
}

type ImportStmt struct {
	Name string
}

type Embedded struct {
	Lang string
	Code string
}

type WriteStmt struct {
	Expr Node
}
