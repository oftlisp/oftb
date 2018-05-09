use std::collections::HashSet;
use std::io::Read;

use failure::Error;
use podio::{LittleEndian, ReadPodExt};

use flatanf::{AExpr, CExpr, Expr, Literal, Program};

type Result<T> = ::std::result::Result<T, Error>;

fn deserialize_u64_as_usize<R: Read>(r: &mut R) -> Result<usize> {
    match r.read_u64::<LittleEndian>() {
        Ok(n) => {
            if n > ::std::usize::MAX as u64 {
                bail!("Overflow in deserializing usize from {}", n)
            } else {
                Ok(n as usize)
            }
        }
        Err(err) => Err(Error::from(err)),
    }
}

fn deserialize_string<R: Read>(r: &mut R) -> Result<String> {
    let len = deserialize_u64_as_usize(r)?;
    let mut buf = vec![0; len];
    r.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(Error::from)
}

impl Program {
    /// Reads the program in from the given Read.
    pub fn deserialize_from<R: Read>(r: &mut R) -> Result<Program> {
        let mut sig = [0; 4];
        r.read_exact(&mut sig)?;
        if &sig != b"ofta" {
            bail!("Invalid signature: {:?}", sig)
        }

        let intrinsics_len = deserialize_u64_as_usize(r)?;
        let mut intrinsics = HashSet::with_capacity(intrinsics_len);
        for _ in 0..intrinsics_len {
            intrinsics.insert(deserialize_string(r)?.into());
        }

        let decls_len = deserialize_u64_as_usize(r)?;
        let mut decls = Vec::with_capacity(decls_len);
        for _ in 0..decls_len {
            let name = deserialize_string(r)?;
            let expr = Expr::deserialize_from(r)?;
            decls.push((name.into(), expr));
        }

        Ok(Program { decls, intrinsics })
    }
}

impl Expr {
    fn deserialize_from<R: Read>(r: &mut R) -> Result<Expr> {
        let discrim = r.read_u8()?;
        match discrim {
            0x00 => {
                let a = Expr::deserialize_from(r)?;
                let b = Expr::deserialize_from(r)?;
                Ok(Expr::Let(Box::new(a), Box::new(b)))
            }
            0x01 => {
                let a = Expr::deserialize_from(r)?;
                let b = Expr::deserialize_from(r)?;
                Ok(Expr::Seq(Box::new(a), Box::new(b)))
            }
            0x02...0x04 => {
                CExpr::deserialize_from_discrim(r, discrim).map(Expr::CExpr)
            }
            0x05...0x09 => {
                AExpr::deserialize_from_discrim(r, discrim).map(Expr::AExpr)
            }
            _ => bail!("Unknown discriminant for Expr: {}", discrim),
        }
    }
}

impl CExpr {
    fn deserialize_from_discrim<R: Read>(
        r: &mut R,
        discrim: u8,
    ) -> Result<CExpr> {
        match discrim {
            0x02 => {
                let func = AExpr::deserialize_from(r)?;
                let argn = deserialize_u64_as_usize(r)?;
                let mut args = Vec::with_capacity(argn);
                for _ in 0..argn {
                    args.push(AExpr::deserialize_from(r)?);
                }
                Ok(CExpr::Call(func, args))
            }
            0x03 => {
                let c = AExpr::deserialize_from(r)?;
                let t = Expr::deserialize_from(r)?;
                let e = Expr::deserialize_from(r)?;
                Ok(CExpr::If(c, Box::new(t), Box::new(e)))
            }
            0x04 => {
                let len = deserialize_u64_as_usize(r)?;
                let mut bound = Vec::new();
                for _ in 0..len {
                    bound.push(AExpr::deserialize_from(r)?);
                }
                let body = Expr::deserialize_from(r)?;
                Ok(CExpr::LetRec(bound, Box::new(body)))
            }
            _ => bail!("Unknown discriminant for CExpr: {}", discrim),
        }
    }
}

impl AExpr {
    fn deserialize_from<R: Read>(r: &mut R) -> Result<AExpr> {
        let discrim = r.read_u8()?;
        AExpr::deserialize_from_discrim(r, discrim)
    }
    fn deserialize_from_discrim<R: Read>(
        r: &mut R,
        discrim: u8,
    ) -> Result<AExpr> {
        match discrim {
            0x05 => {
                let name = deserialize_string(r)?;
                Ok(AExpr::Global(name.into()))
            }
            0x06 => {
                let argn = deserialize_u64_as_usize(r)?;
                let body = Expr::deserialize_from(r)?;
                Ok(AExpr::Lambda(argn, Box::new(body)))
            }
            0x07 => {
                let lit = Literal::deserialize_from(r)?;
                Ok(AExpr::Literal(lit))
            }
            0x08 => {
                let n = deserialize_u64_as_usize(r)?;
                Ok(AExpr::Local(n))
            }
            0x09 => {
                let len = deserialize_u64_as_usize(r)?;
                let mut vals = Vec::with_capacity(len);
                for _ in 0..len {
                    vals.push(AExpr::deserialize_from(r)?);
                }
                Ok(AExpr::Vector(vals))
            }
            _ => bail!("Unknown discriminant for AExpr: {}", discrim),
        }
    }
}

impl Literal {
    fn deserialize_from<R: Read>(r: &mut R) -> Result<Literal> {
        let discrim = r.read_u8()?;
        match discrim {
            0x00 => {
                let n = r.read_u8()?;
                Ok(Literal::Byte(n))
            }
            0x01 => {
                let len = deserialize_u64_as_usize(r)?;
                let mut bs = vec![0; len];
                r.read_exact(&mut bs)?;
                Ok(Literal::Bytes(bs))
            }
            0x02 => {
                let hd = Literal::deserialize_from(r)?;
                let tl = Literal::deserialize_from(r)?;
                Ok(Literal::Cons(Box::new(hd), Box::new(tl)))
            }
            0x03 => {
                let n = deserialize_u64_as_usize(r)?;
                Ok(Literal::Fixnum(n))
            }
            0x04 => Ok(Literal::Nil),
            0x05 => {
                let s = deserialize_string(r)?;
                Ok(Literal::String(s))
            }
            0x06 => {
                let s = deserialize_string(r)?;
                Ok(Literal::Symbol(s.into()))
            }
            0x07 => {
                let len = deserialize_u64_as_usize(r)?;
                let mut vals = Vec::with_capacity(len);
                for _ in 0..len {
                    vals.push(Literal::deserialize_from(r)?);
                }
                Ok(Literal::Vector(vals))
            }
            _ => bail!("Unknown discriminant for Literal: {}", discrim),
        }
    }
}
