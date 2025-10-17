// internal/parser/parser.go - Parser implementation

package parser

import (
	"fmt"
	"regexp"
	"strconv"
	"strings"

	"nula-backend/internal/ast"
)

type Parser struct {
	tokens []Token
	pos    int
}

type Token struct {
	Type  TokenType
	Value string
}

type TokenType int

const (
	TokenIdent TokenType = iota
	TokenNumber
	TokenString
	TokenOperator
	TokenKeyword
	TokenSymbol
	TokenEof
)

var keywords = map[string]bool{
	"if":    true,
	"else":  true,
	"while": true,
	"for":   true,
	"fn":    true,
	"var":   true,
	"write": true,
}

func tokenize(code string) []Token {
	var tokens []Token
	lines := strings.Split(code, "\n")
	inMultiComment := false

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if trimmed == "" {
			continue
		}

		if inMultiComment {
			if strings.Contains(trimmed, "!") {
				inMultiComment = false
			}
			continue
		}

		if strings.HasPrefix(trimmed, "!") {
			inMultiComment = true
			continue
		}

		if strings.HasPrefix(trimmed, "@") {
			continue
		}

		// Tokenize line
		i := 0
		for i < len(trimmed) {
			c := trimmed[i]
			if isLetter(c) {
				id := ""
				for i < len(trimmed) && (isLetter(trimmed[i]) || isDigit(trimmed[i]) || trimmed[i] == '_') {
					id += string(trimmed[i])
					i++
				}
				if keywords[id] {
					tokens = append(tokens, Token{TokenKeyword, id})
				} else {
					tokens = append(tokens, Token{TokenIdent, id})
				}
				continue
			}
			if isDigit(c) || (c == '.' && i+1 < len(trimmed) && isDigit(trimmed[i+1])) {
				num := ""
				for i < len(trimmed) && (isDigit(trimmed[i]) || trimmed[i] == '.') {
					num += string(trimmed[i])
					i++
				}
				tokens = append(tokens, Token{TokenNumber, num})
				continue
			}
			if c == '"' {
				i++
				str := ""
				for i < len(trimmed) && trimmed[i] != '"' {
					str += string(trimmed[i])
					i++
				}
				i++ // skip closing "
				tokens = append(tokens, Token{TokenString, str})
				continue
			}
			if strings.Contains("+-*/^=<>!&|", string(c)) {
				tokens = append(tokens, Token{TokenOperator, string(c)})
				i++
				continue
			}
			if strings.Contains("(){}[]:;,", string(c)) {
				tokens = append(tokens, Token{TokenSymbol, string(c)})
				i++
				continue
			}
			if c == ':' && i+1 < len(trimmed) && trimmed[i+1] == ':' {
				importName := strings.TrimPrefix(trimmed[i+2:], ":")
				tokens = append(tokens, Token{TokenKeyword, "import"})
				tokens = append(tokens, Token{TokenIdent, importName})
				break // Assume whole line
			}
			if c == '<' && strings.HasSuffix(trimmed, ">") {
				module := strings.Trim(trimmed, "<>")
				tokens = append(tokens, Token{TokenKeyword, "from"})
				tokens = append(tokens, Token{TokenIdent, module})
				break
			}
			embeddedRe := regexp.MustCompile(`# =(\w+)= \[(.*)\]`)
			if match := embeddedRe.FindStringSubmatch(trimmed); match != nil {
				tokens = append(tokens, Token{TokenKeyword, "embedded"})
				tokens = append(tokens, Token{TokenIdent, match[1]})
				tokens = append(tokens, Token{TokenString, match[2]})
				break
			}
			i++
		}
	}
	tokens = append(tokens, Token{TokenEof, ""})
	return tokens
}

func Parse(code string) (ast.Program, error) {
	p := &Parser{tokens: tokenize(code)}
	return ast.Program{Statements: p.parseProgram()}, nil
}

func (p *Parser) parseProgram() []ast.Node {
	var stmts []ast.Node
	for p.current() != TokenEof {
		stmts = append(stmts, p.parseStmt())
	}
	return stmts
}

func (p *Parser) parseStmt() ast.Node {
	switch p.currentType() {
	case TokenKeyword:
		switch p.currentValue() {
		case "var":
			return p.parseVarDecl()
		case "if":
			return p.parseIf()
		case "while":
			return p.parseWhile()
		case "for":
			return p.parseFor()
		case "fn":
			return p.parseFuncDef()
		case "write":
			return p.parseWrite()
		case "import":
			return p.parseImport()
		case "from":
			return p.parseFrom()
		case "embedded":
			return p.parseEmbedded()
		}
	case TokenIdent:
		return p.parseAssignOrCall()
	}
	return p.parseExpr()
}

func (p *Parser) parseVarDecl() ast.Node {
	p.advance() // var
	name := p.expect(TokenIdent).Value
	p.expectOp("=")
	value := p.parseExpr()
	return &ast.VarDecl{Name: name, Value: value}
}

func (p *Parser) parseAssignOrCall() ast.Node {
	name := p.expect(TokenIdent).Value
	if p.currentValue() == "=" {
		p.advance()
		return &ast.Assign{Name: name, Value: p.parseExpr()}
	}
	if p.currentValue() == "(" {
		return &ast.FuncCall{Name: name, Args: p.parseArgs()}
	}
	return &ast.Var{Name: name}
}

func (p *Parser) parseIf() ast.Node {
	p.advance() // if
	cond := p.parseExpr()
	p.expectSym("{")
	then := p.parseBlock()
	p.expectSym("}")
	var els []ast.Node
	if p.currentValue() == "else" {
		p.advance()
		p.expectSym("{")
		els = p.parseBlock()
		p.expectSym("}")
	}
	return &ast.IfStmt{Condition: cond, Then: then, Else: els}
}

func (p *Parser) parseWhile() ast.Node {
	p.advance() // while
	cond := p.parseExpr()
	p.expectSym("{")
	body := p.parseBlock()
	p.expectSym("}")
	return &ast.WhileStmt{Condition: cond, Body: body}
}

func (p *Parser) parseFor() ast.Node {
	p.advance() // for
	varName := p.expect(TokenIdent).Value
	p.expectKeyword("in")
	start := p.parseExpr()
	p.expectOp("..")
	end := p.parseExpr()
	p.expectSym("{")
	body := p.parseBlock()
	p.expectSym("}")
	return &ast.ForStmt{Var: varName, Start: start, End: end, Body: body}
}

func (p *Parser) parseFuncDef() ast.Node {
	p.advance() // fn
	name := p.expect(TokenIdent).Value
	p.expectSym("(")
	var params []string
	for p.current() != TokenSymbol || p.currentValue() != ")" {
		params = append(params, p.expect(TokenIdent).Value)
		if p.currentValue() == "," {
			p.advance()
		}
	}
	p.expectSym(")")
	p.expectSym("{")
	body := p.parseBlock()
	p.expectSym("}")
	return &ast.FuncDef{Name: name, Params: params, Body: body}
}

func (p *Parser) parseWrite() ast.Node {
	p.advance() // write
	return &ast.WriteStmt{Expr: p.parseExpr()}
}

func (p *Parser) parseImport() ast.Node {
	p.advance() // import
	name := p.expect(TokenIdent).Value
	return &ast.ImportStmt{Name: name}
}

func (p *Parser) parseFrom() ast.Node {
	p.advance() // from
	name := p.expect(TokenIdent).Value
	// For simplicity, treat as import
	return &ast.ImportStmt{Name: name}
}

func (p *Parser) parseEmbedded() ast.Node {
	p.advance() // embedded
	lang := p.expect(TokenIdent).Value
	code := p.expect(TokenString).Value
	return &ast.Embedded{Lang: lang, Code: code}
}

func (p *Parser) parseBlock() []ast.Node {
	var block []ast.Node
	for p.currentValue() != "}" && p.current() != TokenEof {
		block = append(block, p.parseStmt())
	}
	return block
}

func (p *Parser) parseExpr() ast.Node {
	return p.parseAdd()
}

func (p *Parser) parseAdd() ast.Node {
	left := p.parseMul()
	for p.isOp("+") || p.isOp("-") {
		op := p.advance().Value
		right := p.parseMul()
		left = &ast.BinOp{Op: op, Left: left, Right: right}
	}
	return left
}

func (p *Parser) parseMul() ast.Node {
	left := p.parsePow()
	for p.isOp("*") || p.isOp("/") {
		op := p.advance().Value
		right := p.parsePow()
		left = &ast.BinOp{Op: op, Left: left, Right: right}
	}
	return left
}

func (p *Parser) parsePow() ast.Node {
	left := p.parsePrimary()
	if p.isOp("^") {
		p.advance()
		right := p.parsePrimary()
		left = &ast.BinOp{Op: "^", Left: left, Right: right}
	}
	return left
}

func (p *Parser) parsePrimary() ast.Node {
	switch p.currentType() {
	case TokenNumber:
		val, _ := strconv.ParseFloat(p.advance().Value, 64)
		return &ast.Literal{Value: val}
	case TokenString:
		return &ast.StrLit{Value: p.advance().Value}
	case TokenIdent:
		return p.parseAssignOrCall()
	case TokenSymbol:
		if p.currentValue() == "(" {
			p.advance()
			expr := p.parseExpr()
			p.expectSym(")")
			return expr
		}
	}
	panic(fmt.Sprintf("Unexpected token: %v", p.current()))
}

func (p *Parser) parseArgs() []ast.Node {
	p.expectSym("(")
	var args []ast.Node
	for p.currentValue() != ")" {
		args = append(args, p.parseExpr())
		if p.currentValue() == "," {
			p.advance()
		}
	}
	p.expectSym(")")
	return args
}

func (p *Parser) current() TokenType {
	return p.tokens[p.pos].Type
}

func (p *Parser) currentType() TokenType {
	return p.tokens[p.pos].Type
}

func (p *Parser) currentValue() string {
	return p.tokens[p.pos].Value
}

func (p *Parser) advance() Token {
	tok := p.tokens[p.pos]
	p.pos++
	return tok
}

func (p *Parser) expect(tt TokenType) Token {
	if p.currentType() != tt {
		panic(fmt.Sprintf("Expected %v, got %v", tt, p.currentType()))
	}
	return p.advance()
}

func (p *Parser) expectOp(op string) {
	if !p.isOp(op) {
		panic(fmt.Sprintf("Expected op %s, got %s", op, p.currentValue()))
	}
	p.advance()
}

func (p *Parser) expectSym(sym string) {
	if p.currentValue() != sym {
		panic(fmt.Sprintf("Expected sym %s, got %s", sym, p.currentValue()))
	}
	p.advance()
}

func (p *Parser) expectKeyword(kw string) {
	if p.currentValue() != kw {
		panic(fmt.Sprintf("Expected keyword %s, got %s", kw, p.currentValue()))
	}
	p.advance()
}

func (p *Parser) isOp(op string) bool {
	return p.currentType() == TokenOperator && p.currentValue() == op
}

func isLetter(c byte) bool {
	return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

func isDigit(c byte) bool {
	return c >= '0' && c <= '9'
}
