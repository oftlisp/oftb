use std::io::{Result as IoResult, Write};

use podio::{LittleEndian, WritePodExt};

use flatanf::{AExpr, CExpr, Decl, Expr, Literal, Program};

fn serialize_usize_as_u64<W: Write>(n: usize, w: &mut W) -> IoResult<()> {
    w.write_u64::<LittleEndian>(n as u64)
}

fn serialize_str<W: Write>(s: &str, w: &mut W) -> IoResult<()> {
    serialize_usize_as_u64(s.len(), w)?;
    w.write_all(s.as_bytes())
}

impl Program {
    /// Writes the program out to the given Write.
    pub fn serialize_to<W: Write>(&self, w: &mut W) -> IoResult<()> {
        w.write_all(b"ofta")?;
        serialize_usize_as_u64(self.0.len(), w)?;
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
                w.write_u8(0x00)?;
                val.serialize_to(w)
            }
            Decl::Defn(name, argn, ref body) => {
                serialize_str(name.as_str(), w)?;
                w.write_u8(0x01)?;
                serialize_usize_as_u64(argn, w)?;
                body.serialize_to(w)
            }
        }
    }
}

impl Expr {
    fn serialize_to<W: Write>(&self, w: &mut W) -> IoResult<()> {
        match *self {
            Expr::AExpr(ref e) => e.serialize_to(w),
            Expr::CExpr(ref e) => e.serialize_to(w),
            Expr::Let(ref a, ref b) => {
                w.write_u8(0x00)?;
                a.serialize_to(w)?;
                b.serialize_to(w)
            }
            Expr::Seq(ref a, ref b) => {
                w.write_u8(0x01)?;
                a.serialize_to(w)?;
                b.serialize_to(w)
            }
        }
    }
}

impl CExpr {
    fn serialize_to<W: Write>(&self, w: &mut W) -> IoResult<()> {
        match *self {
            CExpr::Call(ref func, ref args) => {
                w.write_u8(0x02)?;
                func.serialize_to(w)?;
                serialize_usize_as_u64(args.len(), w)?;
                for a in args {
                    a.serialize_to(w)?;
                }
                Ok(())
            }
            CExpr::If(ref c, ref t, ref e) => {
                w.write_u8(0x03)?;
                c.serialize_to(w)?;
                t.serialize_to(w)?;
                e.serialize_to(w)
            }
            CExpr::LetRec(ref bound, ref body) => {
                w.write_u8(0x04)?;
                serialize_usize_as_u64(bound.len(), w)?;
                for e in bound {
                    e.serialize_to(w)?;
                }
                body.serialize_to(w)
            }
        }
    }
}

impl AExpr {
    fn serialize_to<W: Write>(&self, w: &mut W) -> IoResult<()> {
        match *self {
            AExpr::Global(name) => {
                w.write_u8(0x05)?;
                serialize_str(name.as_str(), w)
            }
            AExpr::Lambda(argn, ref body) => {
                w.write_u8(0x06)?;
                serialize_usize_as_u64(argn, w)?;
                body.serialize_to(w)
            }
            AExpr::Literal(ref lit) => {
                w.write_u8(0x07)?;
                lit.serialize_to(w)
            }
            AExpr::Local(n) => {
                w.write_u8(0x08)?;
                serialize_usize_as_u64(n, w)
            }
            AExpr::Vector(ref vec) => {
                w.write_u8(0x09)?;
                serialize_usize_as_u64(vec.len(), w)?;
                for val in vec {
                    val.serialize_to(w)?;
                }
                Ok(())
            }
        }
    }
}

impl Literal {
    fn serialize_to<W: Write>(&self, w: &mut W) -> IoResult<()> {
        match *self {
            Literal::Byte(n) => {
                w.write_u8(0x00)?;
                w.write_u8(n)
            }
            Literal::Bytes(ref bs) => {
                w.write_u8(0x01)?;
                serialize_usize_as_u64(bs.len(), w)?;
                w.write_all(bs)
            }
            Literal::Cons(ref hd, ref tl) => {
                w.write_u8(0x02)?;
                hd.serialize_to(w)?;
                tl.serialize_to(w)
            }
            Literal::Fixnum(n) => {
                w.write_u8(0x03)?;
                serialize_usize_as_u64(n, w)
            }
            Literal::Nil => w.write_u8(0x04),
            Literal::String(ref s) => {
                w.write_u8(0x05)?;
                serialize_str(s, w)
            }
            Literal::Symbol(sym) => {
                w.write_u8(0x06)?;
                serialize_str(sym.as_str(), w)
            }
            Literal::Vector(ref v) => {
                w.write_u8(0x07)?;
                serialize_usize_as_u64(v.len(), w)?;
                for val in v {
                    val.serialize_to(w)?;
                }
                Ok(())
            }
        }
    }
}
