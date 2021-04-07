use std::collections::HashMap;
use crate::types::{Value, ExpandedBuiltIn, UnrolledExpr, MelExpr, VarId, HeapPos};

pub struct MemoryMap {
    memory_store: HashMap<VarId, HeapPos>,
}

impl MemoryMap {
    pub fn new() -> Self {
        MemoryMap { memory_store: HashMap::new() }
    }

    // Abstraction for repetition
    fn binop<F>(&mut self, e1: UnrolledExpr, e2: UnrolledExpr, op: F)
    -> ExpandedBuiltIn<MelExpr>
        where F: Fn(MelExpr, MelExpr) -> ExpandedBuiltIn<MelExpr>
    {
        let mel_e1 = self.to_mel_expr(e1);
        let mel_e2 = self.to_mel_expr(e2);
        op(mel_e1, mel_e2)
    }

    /// Translate an [UnrolledExpr] into a set of low-level [MelExpr] instructions.
    pub fn to_mel_expr(&mut self, expr: UnrolledExpr) -> MelExpr {
        match expr {
            UnrolledExpr::Value(v) => match v {
                Value::Int(n) => MelExpr::Value(Value::Int(n)),
                Value::Bytes(b) => MelExpr::Value(Value::Bytes(b)),
            }
            // A variable by itself is the value of its location in memory
            UnrolledExpr::Var(ref v) => {
                let loc = self.memory_store.get(v)
                              .expect("Expected to find a mapping for variable.");
                MelExpr::BuiltIn(Box::new(ExpandedBuiltIn::Load(*loc)))
            },
            UnrolledExpr::BuiltIn(b) => {
                let mel_b = match *b {
                    ExpandedBuiltIn::Vempty => ExpandedBuiltIn::<MelExpr>::Vempty,
                    ExpandedBuiltIn::Not(e) => ExpandedBuiltIn::<MelExpr>::Not(self.to_mel_expr(e)),
                    ExpandedBuiltIn::Vlen(e) => ExpandedBuiltIn::<MelExpr>::Vlen(self.to_mel_expr(e)),
                    ExpandedBuiltIn::Add(e1,e2) => self.binop(e1,e2, ExpandedBuiltIn::<MelExpr>::Add),
                    ExpandedBuiltIn::Sub(e1,e2) => self.binop(e1,e2, ExpandedBuiltIn::<MelExpr>::Sub),
                    ExpandedBuiltIn::Mul(e1,e2) => self.binop(e1,e2, ExpandedBuiltIn::<MelExpr>::Mul),
                    ExpandedBuiltIn::Div(e1,e2) => self.binop(e1,e2, ExpandedBuiltIn::<MelExpr>::Div),
                    ExpandedBuiltIn::Rem(e1,e2) => self.binop(e1,e2, ExpandedBuiltIn::<MelExpr>::Rem),
                    ExpandedBuiltIn::And(e1,e2) => self.binop(e1,e2, ExpandedBuiltIn::<MelExpr>::And),
                    ExpandedBuiltIn::Or(e1,e2) => self.binop(e1,e2, ExpandedBuiltIn::<MelExpr>::Or),
                    ExpandedBuiltIn::Xor(e1,e2) => self.binop(e1,e2, ExpandedBuiltIn::<MelExpr>::Xor),
                    ExpandedBuiltIn::Vpush(e1,e2) => self.binop(e1,e2, ExpandedBuiltIn::<MelExpr>::Vpush),
                    ExpandedBuiltIn::Vappend(e1,e2) => self.binop(e1,e2, ExpandedBuiltIn::<MelExpr>::Vappend),
                    ExpandedBuiltIn::Vref(e1,e2) => self.binop(e1,e2, ExpandedBuiltIn::<MelExpr>::Vref),
                    _ => unreachable!(),
                };

                MelExpr::BuiltIn(Box::new(mel_b))
            },
            UnrolledExpr::Set(var_id, body) => {
                let mel_body = self.to_mel_expr(*body);
                let loc = self.memory_store.get(&var_id)
                              .expect("Failed to access variable id, there's a bug somewhere.");

                // Evaluate the body, then store the result in memory at `loc`
                MelExpr::Seq(vec![
                    mel_body,
                    MelExpr::BuiltIn(Box::new(ExpandedBuiltIn::Store(loc.clone())))])
            },
            UnrolledExpr::If(pred, on_true, on_false) => {
                let mel_true = self.to_mel_expr(*on_true);
                let mel_false = self.to_mel_expr(*on_false);

                MelExpr::Seq(vec![
                    self.to_mel_expr(*pred),
                    MelExpr::BuiltIn(Box::new(ExpandedBuiltIn::Bez((count_insts(&mel_true) + 1) as u16))),
                    mel_true,
                    MelExpr::BuiltIn(Box::new(ExpandedBuiltIn::Jmp(count_insts(&mel_false) as u16))),
                    mel_false,
                ])
            },
            UnrolledExpr::Let(binds, exprs) => {
                // For each binding, evaluate the expression (to push onto stack) and store in a new
                // memory location.
                // TODO: What happens when the binding expression is a 'set!'?
                let mut mel_binds = vec![];
                binds.into_iter().for_each(|(var_id, expr)| {
                    // Make sure the variable is not somehow already there
                    //self.memory_store.get(&var_id)
                        //.expect("Variable id in let binding should not already be defined, this is a bug.");

                    // Assign the variable a memory location
                    // TODO: For simplicity, just converting the id into an address. This
                    // should probably be decoupled though.
                    let loc = var_id as HeapPos;
                    self.memory_store.insert(var_id, loc);

                    // Translate expr into mel instructions
                    let mel_expr = self.to_mel_expr(expr);

                    // Evaluate the expression,
                    // then store whatever is popped from the stack at 'loc'
                    mel_binds.push(mel_expr);
                    mel_binds.push( MelExpr::BuiltIn(Box::new(ExpandedBuiltIn::Store(loc))) );
                });

                // Finally, evaluate the body
                let mel_exprs = exprs.into_iter().map(|e| self.to_mel_expr(e));
                //mel_binds.push(mel_body);
                mel_binds.extend(mel_exprs);

                MelExpr::Seq( mel_binds )
            },
            UnrolledExpr::Loop(n, expr) => MelExpr::Loop(n, Box::new(self.to_mel_expr(*expr))),
            UnrolledExpr::Hash(n, expr) => MelExpr::Hash(n, Box::new(self.to_mel_expr(*expr))),
            UnrolledExpr::Sigeok(n, e1,e2,e3) =>
                MelExpr::Sigeok(n, Box::new(self.to_mel_expr(*e1)),
                                   Box::new(self.to_mel_expr(*e2)),
                                   Box::new(self.to_mel_expr(*e3))),
        }
    }
}

/// Count the number of primitive instructions recursively from a [MelExpr].
pub fn count_insts(e: &MelExpr) -> u16 {
    match e {
        MelExpr::Seq(v) => v.iter().map(count_insts).reduce(|a,b| a+b).unwrap_or(0),
        MelExpr::Loop(_,e) => 1 + count_insts(e),
        MelExpr::Hash(_,e) => 1 + count_insts(e),
        MelExpr::Sigeok(_,e1,e2,e3) => 1 + count_insts(e1) + count_insts(e2) + count_insts(e3),
        MelExpr::Value(val) => match val {
            //Value::Vec(v) => v.len(),
            Value::Int(_) => 1,
            Value::Bytes(_) => 1,
        },
        MelExpr::BuiltIn(b) => match &**b {
            ExpandedBuiltIn::Add(e1,e2) => 1 + count_insts(&e1) + count_insts(&e2),
            ExpandedBuiltIn::Sub(e1,e2) => 1 + count_insts(&e1) + count_insts(&e2),
            ExpandedBuiltIn::Mul(e1,e2) => 1 + count_insts(&e1) + count_insts(&e2),
            ExpandedBuiltIn::Div(e1,e2) => 1 + count_insts(&e1) + count_insts(&e2),
            ExpandedBuiltIn::Rem(e1,e2) => 1 + count_insts(&e1) + count_insts(&e2),
            ExpandedBuiltIn::And(e1,e2) => 1 + count_insts(&e1) + count_insts(&e2),
            ExpandedBuiltIn::Or(e1,e2)  => 1 + count_insts(&e1) + count_insts(&e2),
            ExpandedBuiltIn::Xor(e1,e2) => 1 + count_insts(&e1) + count_insts(&e2),
            ExpandedBuiltIn::Not(e)     => 1 + count_insts(&e),
            ExpandedBuiltIn::Vref(e1,e2)    => 1 + count_insts(&e1) + count_insts(&e2),
            ExpandedBuiltIn::Vappend(e1,e2) => 1 + count_insts(&e1) + count_insts(&e2),
            ExpandedBuiltIn::Vempty         => 1,
            ExpandedBuiltIn::Vlen(e)        => 1 + count_insts(&e),
            ExpandedBuiltIn::Vpush(e1,e2)   => 1 + count_insts(&e1) + count_insts(&e2),
            ExpandedBuiltIn::Vslice(e1,e2,e3) => 1 + count_insts(&e1) + count_insts(&e2) + count_insts(&e3),
            ExpandedBuiltIn::Jmp(n)     => 1,
            ExpandedBuiltIn::Bez(n)     => 1,
            ExpandedBuiltIn::Bnz(n)     => 1,
            ExpandedBuiltIn::Loop(n, e) => 1 + count_insts(&e),
            ExpandedBuiltIn::Store(idx) => 1,
            ExpandedBuiltIn::Load(idx)  => 1,
        },
    }
}
