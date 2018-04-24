use std::io::{Result as IoResult, Write};

use podio::{LittleEndian, WritePodExt};

use flatanf::{AExpr, CExpr, Decl, Expr, Literal, Program};

fn serialize_as_u32<W: Write>(n: usize, w: &mut W) -> IoResult<()> {
    assert!(n <= ::std::u32::MAX as usize);
    w.write_u32::<LittleEndian>(n as u32)
}

fn serialize_str<W: Write>(s: &str, w: &mut W) -> IoResult<()> {
    serialize_as_u32(s.len(), w);
    w.write_all(s.as_bytes())
}

impl Program {
    /// Writes the program out to the given Write.
    pub fn serialize_to<W: Write>(&self, w: &mut W) -> IoResult<()> {
        w.write_all(b"ofta")?;
        serialize_as_u32(self.0.len(), w)?;
        for decl in &self.0 {
            decl.serialize_to(w)?;
        }
        Ok(())
    }
}

impl Decl {
    fn serialize_to<W: Write>(&self, w: &mut W) -> IoResult<()> {
        match *self {
            Decl::Def(name, ref val) => {
                serialize_str(name.as_str(), w)?;
                w.write_u32::<LittleEndian>(0)?;
                val.serialize_to(w)
            }
            Decl::Defn(name, argn, ref body) => {
                serialize_str(name.as_str(), w)?;
                assert_ne!(argn, 0);
                serialize_as_u32(argn, w)?;
                body.serialize_to(w)
            }
        }
    }
}

impl Expr {
    fn serialize_to<W: Write>(&self, w: &mut W) -> IoResult<()> {
        match *self {
            Expr::AExpr(ref e) => unimplemented!(),
            Expr::CExpr(ref e) => unimplemented!(),
            Expr::Let(ref a, ref b) => unimplemented!(),
            Expr::Seq(ref a, ref b) => unimplemented!(),
        }
    }
}

impl CExpr {
    fn serialize_to<W: Write>(&self, w: &mut W) -> IoResult<()> {
        match *self {
            CExpr::Call(ref func, ref args) => unimplemented!(),
            CExpr::If(ref c, ref t, ref e) => unimplemented!(),
            CExpr::LetRec(ref bound, ref body) => unimplemented!(),
        }
    }
}

impl AExpr {
    fn serialize_to<W: Write>(&self, w: &mut W) -> IoResult<()> {
        match *self {
            AExpr::Global(name) => unimplemented!(),
            AExpr::Lambda(argn, ref body) => unimplemented!(),
            AExpr::Literal(ref lit) => unimplemented!(),
            AExpr::Local(n) => unimplemented!(),
            AExpr::Vector(ref vec) => unimplemented!(),
        }
    }
}

impl Literal {
    fn serialize_to<W: Write>(&self, w: &mut W) -> IoResult<()> {
        match *self {
            Literal::Byte(n) => unimplemented!(),
            Literal::Bytes(ref bs) => unimplemented!(),
            Literal::Cons(ref hd, ref tl) => unimplemented!(),
            Literal::Fixnum(n) => unimplemented!(),
            Literal::Nil => unimplemented!(),
            Literal::String(ref str) => unimplemented!(),
            Literal::Symbol(sym) => unimplemented!(),
            Literal::Vector(ref v) => unimplemented!(),
        }
    }
}
