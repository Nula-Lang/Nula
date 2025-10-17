// internal/interpreter/interpreter.go - Interpreter logic

package interpreter

import (
	"fmt"
	"math"
	"os"
	"os/exec"

	"nula-backend/internal/ast"
)

type Scope struct {
	vars   map[string]interface{}
	parent *Scope
	funcs  map[string]*ast.FuncDef
}

func NewScope(parent *Scope) *Scope {
	return &Scope{
		vars:   make(map[string]interface{}),
		parent: parent,
		funcs:  make(map[string]*ast.FuncDef),
	}
}

func (s *Scope) Get(name string) interface{} {
	if val, ok := s.vars[name]; ok {
		return val
	}
	if s.parent != nil {
		return s.parent.Get(name)
	}
	return nil
}

func (s *Scope) Set(name string, val interface{}) {
	s.vars[name] = val
}

func (s *Scope) DefineFunc(def *ast.FuncDef) {
	s.funcs[def.Name] = def
}

func (s *Scope) GetFunc(name string) *ast.FuncDef {
	if f, ok := s.funcs[name]; ok {
		return f
	}
	if s.parent != nil {
		return s.parent.GetFunc(name)
	}
	return nil
}

func Interpret(program ast.Program, scope *Scope) error {
	for _, stmt := range program.Statements {
		_, err := eval(stmt, scope)
		if err != nil {
			return err
		}
	}
	return nil
}

func eval(node ast.Node, scope *Scope) (interface{}, error) {
	switch n := node.(type) {
	case *ast.Program:
		var last interface{}
		for _, stmt := range n.Statements {
			var err error
			last, err = eval(stmt, scope)
			if err != nil {
				return nil, err
			}
		}
		return last, nil
	case *ast.VarDecl:
		val, err := eval(n.Value, scope)
		if err != nil {
			return nil, err
		}
		scope.Set(n.Name, val)
		return val, nil
	case *ast.Assign:
		val, err := eval(n.Value, scope)
		if err != nil {
			return nil, err
		}
		scope.Set(n.Name, val)
		return val, nil
	case *ast.IfStmt:
		cond, err := eval(n.Condition, scope)
		if err != nil {
			return nil, err
		}
		if isTrue(cond) {
			for _, stmt := range n.Then {
				_, err := eval(stmt, scope)
				if err != nil {
					return nil, err
				}
			}
		} else {
			for _, stmt := range n.Else {
				_, err := eval(stmt, scope)
				if err != nil {
					return nil, err
				}
			}
		}
		return nil, nil
	case *ast.WhileStmt:
		for {
			cond, err := eval(n.Condition, scope)
			if err != nil {
				return nil, err
			}
			if !isTrue(cond) {
				break
			}
			for _, stmt := range n.Body {
				_, err := eval(stmt, scope)
				if err != nil {
					return nil, err
				}
			}
		}
		return nil, nil
	case *ast.ForStmt:
		start, err := eval(n.Start, scope)
		if err != nil {
			return nil, err
		}
		end, err := eval(n.End, scope)
		if err != nil {
			return nil, err
		}
		s, e := toFloat(start), toFloat(end)
		for i := s; i < e; i++ {
			scope.Set(n.Var, i)
			for _, stmt := range n.Body {
				_, err := eval(stmt, scope)
				if err != nil {
					return nil, err
				}
			}
		}
		return nil, nil
	case *ast.FuncDef:
		scope.DefineFunc(n)
		return nil, nil
	case *ast.FuncCall:
		f := scope.GetFunc(n.Name)
		if f == nil {
			return nil, fmt.Errorf("undefined function %s", n.Name)
		}
		args := make([]interface{}, len(n.Args))
		for i, arg := range n.Args {
			val, err := eval(arg, scope)
			if err != nil {
				return nil, err
			}
			args[i] = val
		}
		funcScope := NewScope(scope)
		for i, param := range f.Params {
			funcScope.Set(param, args[i])
		}
		for _, stmt := range f.Body {
			_, err := eval(stmt, funcScope)
			if err != nil {
				return nil, err
			}
		}
		return nil, nil // Assume no return for now
	case *ast.BinOp:
		left, err := eval(n.Left, scope)
		if err != nil {
			return nil, err
		}
		right, err := eval(n.Right, scope)
		if err != nil {
			return nil, err
		}
		l, r := toFloat(left), toFloat(right)
		switch n.Op {
		case "+":
			return l + r, nil
		case "-":
			return l - r, nil
		case "*":
			return l * r, nil
		case "/":
			return l / r, nil
		case "^":
			return math.Pow(l, r), nil
		}
	case *ast.Literal:
		return n.Value, nil
	case *ast.StrLit:
		return n.Value, nil
	case *ast.Var:
		val := scope.Get(n.Name)
		if val == nil {
			return nil, fmt.Errorf("undefined variable %s", n.Name)
		}
		return val, nil
	case *ast.ImportStmt:
		fmt.Printf("Imported %s\n", n.Name)
		return nil, nil
	case *ast.Embedded:
		switch n.Lang {
		case "python":
			tmpFile, err := os.CreateTemp("", "embedded*.py")
			if err != nil {
				return nil, err
			}
			defer os.Remove(tmpFile.Name())
			_, err = tmpFile.WriteString(n.Code)
			if err != nil {
				return nil, err
			}
			tmpFile.Close()
			cmd := exec.Command("python", tmpFile.Name())
			output, err := cmd.Output()
			if err != nil {
				return nil, err
			}
			fmt.Print(string(output))
		}
		return nil, nil
	case *ast.WriteStmt:
		val, err := eval(n.Expr, scope)
		if err != nil {
			return nil, err
		}
		fmt.Println(val)
		return nil, nil
	}
	return nil, fmt.Errorf("unknown node type")
}

func isTrue(val interface{}) bool {
	switch v := val.(type) {
	case float64:
		return v != 0
	case string:
		return v != ""
	case bool:
		return v
	}
	return true
}

func toFloat(val interface{}) float64 {
	switch v := val.(type) {
	case float64:
		return v
	case int:
		return float64(v)
	case string:
		f, _ := strconv.ParseFloat(v, 64)
		return f
	}
	return 0
}
