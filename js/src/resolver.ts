import { Everest } from './everest';
import type { Expr } from './expr';
import type { Interpreter } from './interpreter';
import type { Stmt } from './stmt';
import type { Token } from './token';

export class Resolver implements Expr.Visitor<void>, Stmt.Visitor<void> {
  private readonly interpreter: Interpreter;
  private readonly scopes: Array<Map<string, boolean>> = [];
  private current_fn: FnKind = FnKind.None;
  private current_class: ClassKind = ClassKind.None;

  constructor(interpreter: Interpreter) {
    this.interpreter = interpreter;
  }

  public resolve(statements: Array<Stmt>): void {
    for (const statement of statements) {
      this.resolve_statement(statement);
    }
  }

  visit_block_stmt(stmt: Stmt.Block): void {
    this.begin_scope();
    this.resolve(stmt.statements);
    this.end_scope();
  }

  visit_class_stmt(stmt: Stmt.Class): void {
    const enclosing_class = this.current_class;
    this.current_class = ClassKind.Class;
    this.declare(stmt.name);
    this.define(stmt.name);

    if (stmt.superclass !== undefined && stmt.name.lexeme === stmt.superclass.name.lexeme) {
      Everest.error_with(stmt.superclass.name, 'class cannot inherit itself');
    }

    if (stmt.superclass !== undefined) {
      this.current_class = ClassKind.Subclass;
      this.resolve_expr(stmt.superclass);
    }

    if (stmt.superclass !== undefined) {
      this.begin_scope();
      this.scopes[this.scopes.length - 1]?.set('super', true);
    }

    this.begin_scope();
    this.scopes[this.scopes.length - 1]?.set('this', true);

    for (const method of stmt.methods) {
      let declaration = FnKind.Method;
      if (method.name.lexeme === 'init') {
        declaration = FnKind.Initializer;
      }

      this.resolve_fn(method, declaration);
    }

    this.end_scope();

    if (stmt.superclass !== undefined) { this.end_scope(); }

    this.current_class = enclosing_class;
  }

  visit_expression_stmt(stmt: Stmt.Expression): void {
    this.resolve_expr(stmt.expression);
  }



  private resolve_statement(stmt: Stmt): void {
    stmt.accept(this);
  }

  private resolve_expr(expr: Expr): void {
    expr.accept(this);
  }

  private resolve_fn(fn: Stmt.Fn, kind: FnKind) {
    const enclosing_fn = this.current_fn;
    this.current_fn = kind;

    this.begin_scope();
    for (const param of fn.params) {
      this.declare(param);
      this.define(param);
    }
    this.end_scope();

    this.resolve(fn.body);

    this.current_fn = enclosing_fn;
  }

  private begin_scope(): void {
    this.scopes.push(new Map());
  }

  private end_scope(): void {
    this.scopes.pop();
  }

  private declare(name: Token): void {
    if (this.scopes.length === 0) {
      return;
    }

    const scope = this.scopes[this.scopes.length - 1] as Map<string, boolean>;
    if (scope.has(name.lexeme)) {
      Everest.error_with(name, 'variable already exists here');
    }

    scope.set(name.lexeme, false);
  }

  private define(name: Token): void {
    if (this.scopes.length === 0) {
      return;
    }

    this.scopes[this.scopes.length - 1]?.set(name.lexeme, true);
  }

  private resolve_local(expr: Expr, name: Token): void {
    for (let i = this.scopes.length - 1; i >= 0; i--) {
      if (this.scopes[i]?.has(name.lexeme)) {
        this.interpreter.resolve(expr, this.scopes.length - 1 - i);
        return;
      }
    }
  }
}

export enum FnKind {
  None,
  Fn,
  Initializer,
  Method,
}

export enum ClassKind {
  None,
  Class,
  Subclass,
}