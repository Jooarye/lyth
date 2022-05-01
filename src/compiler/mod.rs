use super::lexer::token::TokenKind;
use super::parser::{ast, Parser};
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate;
use llvm_sys::{core, target, target_machine};
use std::collections::HashMap;
use std::ffi::{CStr, CString};

pub struct Compiler<'a> {
    file: &'a str,
    input: &'a str,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    function: Option<LLVMValueRef>,
    function_type: Option<ast::Type>,
    named_values: HashMap<String, LLVMValueRef>,
}

impl<'a> Compiler<'a> {
    pub fn new(file: &'a str, input: &'a str) -> Self {
        unsafe {
            if target::LLVM_InitializeNativeTarget() != 0 {
                panic!("Could not initialise target");
            }
            if target::LLVM_InitializeNativeAsmPrinter() != 0 {
                panic!("Could not initialise ASM Printer");
            }
        }
        let mod_name = CStr::from_bytes_with_nul(b"lyth-compiled-module\0").unwrap();

        Self {
            file,
            input,
            module: unsafe { core::LLVMModuleCreateWithName(mod_name.as_ptr()) },
            builder: unsafe { core::LLVMCreateBuilder() },
            function: None,
            function_type: None,
            named_values: HashMap::new(),
        }
    }

    pub fn compile(&mut self, path: &str) {
        let ast = Parser::new(self.file, self.input).parse();

        for decl in ast.iter() {
            match decl {
                ast::Decl::Function {
                    name,
                    params,
                    body,
                    rtyp,
                } => {
                    self.named_values.clear();
                    let function_type = unsafe {
                        let mut param_types = Vec::new();

                        for (_, typ) in params {
                            param_types.push(self.get_type(typ));
                        }

                        let return_type = match rtyp {
                            Some(t) => self.get_type(t),
                            None => core::LLVMVoidType(),
                        };

                        core::LLVMFunctionType(
                            return_type,
                            param_types.as_mut_ptr(),
                            param_types.len() as u32,
                            0,
                        )
                    };

                    let function_name = CString::new(name.as_bytes()).unwrap();
                    self.function = Some(unsafe {
                        core::LLVMAddFunction(self.module, function_name.as_ptr(), function_type)
                    });
                    self.function_type = rtyp.clone();

                    let block_name = CStr::from_bytes_with_nul(b"entry\0").unwrap();
                    let entry_block = unsafe {
                        core::LLVMAppendBasicBlock(self.function.unwrap(), block_name.as_ptr())
                    };

                    unsafe { core::LLVMPositionBuilderAtEnd(self.builder, entry_block) };

                    let mut idx = 0;
                    for (name, typ) in params {
                        let param_name = CStr::from_bytes_with_nul(b"param\0").unwrap();
                        let param_val = unsafe { core::LLVMGetParam(self.function.unwrap(), idx) };
                        let param_at = unsafe {
                            core::LLVMBuildAlloca(
                                self.builder,
                                self.get_type(typ),
                                param_name.as_ptr(),
                            )
                        };

                        unsafe { core::LLVMBuildStore(self.builder, param_val, param_at) };

                        self.named_values.insert(name.clone(), param_at);
                        idx += 1;
                    }

                    self.compile_stmt(body);

                    self.function = None;
                    self.function_type = None;
                }
                _ => panic!("not implemented"),
            }
        }

        unsafe {
            let triple = target_machine::LLVMGetDefaultTargetTriple();
            let mut target: target_machine::LLVMTargetRef =
                std::mem::MaybeUninit::uninit().assume_init();
            let error = std::mem::MaybeUninit::uninit().assume_init();
            target_machine::LLVMGetTargetFromTriple(triple, &mut target, error);

            let cpu = CStr::from_bytes_with_nul(b"generic\0").unwrap();
            let features = CStr::from_bytes_with_nul(b"\0").unwrap();

            let machine = target_machine::LLVMCreateTargetMachine(
                target,
                triple,
                cpu.as_ptr(),
                features.as_ptr(),
                target_machine::LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
                target_machine::LLVMRelocMode::LLVMRelocDefault,
                target_machine::LLVMCodeModel::LLVMCodeModelDefault,
            );

            // core::LLVMDumpModule(self.module);

            let f = CString::new(path.as_bytes()).unwrap();
            target_machine::LLVMTargetMachineEmitToFile(
                machine,
                self.module,
                f.as_ptr() as *mut i8,
                target_machine::LLVMCodeGenFileType::LLVMObjectFile,
                error,
            );
        }
    }

    fn compile_stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::Block { body } => {
                for s in body {
                    self.compile_stmt(s);
                }
            }
            ast::Stmt::Let { name, typ, value } => {
                let variable_type = self.get_type(&typ.clone().unwrap());

                let variable_name = CString::new(name.as_bytes()).unwrap();
                let variable = unsafe {
                    core::LLVMBuildAlloca(self.builder, variable_type, variable_name.as_ptr())
                };

                self.named_values.insert(name.clone(), variable);

                let val = self.compile_expr(value);

                unsafe { core::LLVMBuildStore(self.builder, val, variable) };
            }
            ast::Stmt::If { expr, body, elze } => {
                let cond = self.compile_expr(expr);

                let cons_name = CStr::from_bytes_with_nul(b"cons\0").unwrap();
                let cons_block = unsafe {
                    core::LLVMAppendBasicBlock(self.function.unwrap(), cons_name.as_ptr())
                };

                let alter_name = CStr::from_bytes_with_nul(b"alter\0").unwrap();
                let alter_block = unsafe {
                    core::LLVMAppendBasicBlock(self.function.unwrap(), alter_name.as_ptr())
                };

                let merge_name = CStr::from_bytes_with_nul(b"merge\0").unwrap();
                let merge_block = unsafe {
                    core::LLVMAppendBasicBlock(self.function.unwrap(), merge_name.as_ptr())
                };

                unsafe {
                    core::LLVMBuildCondBr(self.builder, cond, cons_block, alter_block);
                    core::LLVMPositionBuilderAtEnd(self.builder, cons_block);
                }

                self.compile_stmt(body);

                unsafe { core::LLVMBuildBr(self.builder, merge_block) };

                unsafe { core::LLVMPositionBuilderAtEnd(self.builder, alter_block) };
                if elze.is_some() {
                    let elze = elze.clone().unwrap();
                    self.compile_stmt(elze.as_ref());
                }
                unsafe { core::LLVMBuildBr(self.builder, merge_block) };

                unsafe { core::LLVMPositionBuilderAtEnd(self.builder, merge_block) };
            }
            ast::Stmt::Assign { name, op: _, value } => {
                let ptr = self.named_values[name];
                let val = self.compile_expr(value);

                unsafe { core::LLVMBuildStore(self.builder, val, ptr) };
            }
            ast::Stmt::Return { value } => match value {
                Some(val) => {
                    let val = self.compile_expr(val);
                    unsafe { core::LLVMBuildRet(self.builder, val) };
                }
                None => {
                    unsafe { core::LLVMBuildRetVoid(self.builder) };
                }
            },
            _ => panic!("not implemented"),
        }
    }

    fn compile_expr(&mut self, expr: &Box<ast::Expr>) -> LLVMValueRef {
        match expr.as_ref() {
            ast::Expr::Literal(l) => match l {
                ast::Lit::Integer(i) => unsafe {
                    core::LLVMConstInt(core::LLVMInt64Type(), *i as u64, 0)
                },
                ast::Lit::Boolean(b) => unsafe {
                    core::LLVMConstInt(core::LLVMInt1Type(), if *b { 1 } else { 0 }, 0)
                },
                _ => panic!("not implemented"),
            },
            ast::Expr::Call { name, args } => {
                let mut func_args = Vec::new();

                for arg in args {
                    func_args.push(self.compile_expr(arg));
                }

                let func_name = CString::new(name.as_bytes()).unwrap();
                let func = unsafe { core::LLVMGetNamedFunction(self.module, func_name.as_ptr()) };
                let n = CStr::from_bytes_with_nul(b"call\0").unwrap();

                unsafe {
                    core::LLVMBuildCall(
                        self.builder,
                        func,
                        func_args.as_mut_ptr(),
                        func_args.len() as u32,
                        n.as_ptr(),
                    )
                }
            }
            ast::Expr::Prefix { op, expr } => {
                let x = self.compile_expr(expr);

                match op {
                    TokenKind::Minus => unsafe {
                        let name = CStr::from_bytes_with_nul(b"neg\0").unwrap();
                        core::LLVMBuildNeg(self.builder, x, name.as_ptr())
                    },
                    TokenKind::Bang => unsafe {
                        let name = CStr::from_bytes_with_nul(b"not\0").unwrap();
                        core::LLVMBuildNot(self.builder, x, name.as_ptr())
                    },
                    _ => panic!("not a valid prefix operator"),
                }
            }
            ast::Expr::Infix { op, left, right } => {
                let lhs = self.compile_expr(left);
                let rhs = self.compile_expr(right);

                match op {
                    TokenKind::Plus => unsafe {
                        let name = CStr::from_bytes_with_nul(b"add\0").unwrap();
                        core::LLVMBuildAdd(self.builder, lhs, rhs, name.as_ptr())
                    },
                    TokenKind::Minus => unsafe {
                        let name = CStr::from_bytes_with_nul(b"sub\0").unwrap();
                        core::LLVMBuildSub(self.builder, lhs, rhs, name.as_ptr())
                    },
                    TokenKind::Asterisk => unsafe {
                        let name = CStr::from_bytes_with_nul(b"mul\0").unwrap();
                        core::LLVMBuildMul(self.builder, lhs, rhs, name.as_ptr())
                    },
                    TokenKind::Slash => unsafe {
                        let name = CStr::from_bytes_with_nul(b"div\0").unwrap();
                        core::LLVMBuildSDiv(self.builder, lhs, rhs, name.as_ptr())
                    },
                    TokenKind::Percent => unsafe {
                        let name = CStr::from_bytes_with_nul(b"mod\0").unwrap();
                        core::LLVMBuildSRem(self.builder, lhs, rhs, name.as_ptr())
                    },
                    TokenKind::And => unsafe {
                        let name = CStr::from_bytes_with_nul(b"and\0").unwrap();
                        core::LLVMBuildAnd(self.builder, lhs, rhs, name.as_ptr())
                    },
                    TokenKind::Pipe => unsafe {
                        let name = CStr::from_bytes_with_nul(b"or\0").unwrap();
                        core::LLVMBuildOr(self.builder, lhs, rhs, name.as_ptr())
                    },
                    TokenKind::Caret => unsafe {
                        let name = CStr::from_bytes_with_nul(b"xor\0").unwrap();
                        core::LLVMBuildXor(self.builder, lhs, rhs, name.as_ptr())
                    },
                    TokenKind::Equal => unsafe {
                        let name = CStr::from_bytes_with_nul(b"eq\0").unwrap();
                        core::LLVMBuildICmp(
                            self.builder,
                            LLVMIntPredicate::LLVMIntEQ,
                            lhs,
                            rhs,
                            name.as_ptr(),
                        )
                    },
                    TokenKind::UnEqual => unsafe {
                        let name = CStr::from_bytes_with_nul(b"ue\0").unwrap();
                        core::LLVMBuildICmp(
                            self.builder,
                            LLVMIntPredicate::LLVMIntNE,
                            lhs,
                            rhs,
                            name.as_ptr(),
                        )
                    },
                    TokenKind::LessThan => unsafe {
                        let name = CStr::from_bytes_with_nul(b"lt\0").unwrap();
                        core::LLVMBuildICmp(
                            self.builder,
                            LLVMIntPredicate::LLVMIntSLT,
                            lhs,
                            rhs,
                            name.as_ptr(),
                        )
                    },
                    TokenKind::GreaterThan => unsafe {
                        let name = CStr::from_bytes_with_nul(b"gt\0").unwrap();
                        core::LLVMBuildICmp(
                            self.builder,
                            LLVMIntPredicate::LLVMIntSGT,
                            lhs,
                            rhs,
                            name.as_ptr(),
                        )
                    },
                    TokenKind::LessEqual => unsafe {
                        let name = CStr::from_bytes_with_nul(b"le\0").unwrap();
                        core::LLVMBuildICmp(
                            self.builder,
                            LLVMIntPredicate::LLVMIntSLE,
                            lhs,
                            rhs,
                            name.as_ptr(),
                        )
                    },
                    TokenKind::GreaterEqual => unsafe {
                        let name = CStr::from_bytes_with_nul(b"ge\0").unwrap();
                        core::LLVMBuildICmp(
                            self.builder,
                            LLVMIntPredicate::LLVMIntSGE,
                            lhs,
                            rhs,
                            name.as_ptr(),
                        )
                    },
                    _ => panic!("not a valid infix operator"),
                }
            }
            ast::Expr::Ident(name) => unsafe {
                let load_name = CStr::from_bytes_with_nul(b"load\0").unwrap();
                core::LLVMBuildLoad(self.builder, self.named_values[name], load_name.as_ptr())
            },

            _ => panic!("not implemented"),
        }
    }

    fn get_type(&self, typ: &ast::Type) -> LLVMTypeRef {
        unsafe {
            match typ.name.as_ref() {
                "i128" => core::LLVMInt128Type(),
                "i64" => core::LLVMInt64Type(),
                "i32" => core::LLVMInt32Type(),
                "i16" => core::LLVMInt16Type(),
                "i8" => core::LLVMInt8Type(),
                "bool" => core::LLVMInt1Type(),

                _ => panic!("unknown type"),
            }
        }
    }
}
