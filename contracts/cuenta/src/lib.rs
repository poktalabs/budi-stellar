//! Cuenta: la cuenta de un agente, en la línea de ERC-6551 (token bound accounts).
//!
//! Cada agente registrado en `Agentes` puede tener su propia cuenta en Stellar: un
//! contrato con dirección propia que recibe tokens y actúa en otros contratos. No
//! tiene dueño fijo: la controla quien sea dueño del agente en ese momento, así que
//! si el agente cambia de manos, la cuenta (con su saldo y su historial) se va con él.
#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype, token,
    Address, Env, Symbol, Val, Vec,
};

#[contracttype]
enum Clave {
    Agentes,
    Agente,
    Operador,
    Congelada,
    TopeDiario,
    GastadoHoy(u64),
}

/// La llave con la que el agente opera solo (su runtime), con un tope por pago.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Operador {
    pub llave: Address,
    pub token: Address,
    pub tope_por_pago: i128,
}

#[contractclient(name = "AgentesClient")]
pub trait AgentesApi {
    fn dueno(env: Env, id: u32) -> Address;
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    MontoInvalido = 1,
    SinOperador = 2,
    ExcedeTope = 3,
    Congelada = 4,
    NoCongelada = 5,
    ExcedeTopeDiario = 6,
    SinTopeDiario = 7,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pagado {
    #[topic]
    pub token: Address,
    pub a: Address,
    pub monto: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ejecutado {
    #[topic]
    pub contrato: Address,
    pub funcion: Symbol,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PagoEnLote {
    #[topic]
    pub token: Address,
    pub pagos: u32,
    pub total: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GastoAprobado {
    #[topic]
    pub token: Address,
    pub gastador: Address,
    pub monto: i128,
    pub hasta_ledger: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Retirado {
    #[topic]
    pub token: Address,
    pub a: Address,
    pub monto: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GastoRevocado {
    #[topic]
    pub token: Address,
    pub gastador: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperadorFijado {
    #[topic]
    pub llave: Address,
    pub token: Address,
    pub tope_por_pago: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperadorQuitado {
    #[topic]
    pub llave: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PagoDelOperador {
    #[topic]
    pub llave: Address,
    pub a: Address,
    pub monto: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopeDiarioFijado {
    pub por: Address,
    pub tope: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopeDiarioQuitado {
    pub por: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CuentaCongelada {
    pub por: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CuentaDescongelada {
    pub por: Address,
}

#[contract]
pub struct Cuenta;

#[contractimpl]
impl Cuenta {
    pub fn __constructor(env: Env, agentes: Address, agente: u32) {
        env.storage().instance().set(&Clave::Agentes, &agentes);
        env.storage().instance().set(&Clave::Agente, &agente);
    }

    /// El dueño actual del agente paga desde la cuenta del agente.
    pub fn pagar(env: Env, token: Address, a: Address, monto: i128) -> Result<(), Error> {
        Self::dueno(env.clone()).require_auth();
        if monto <= 0 {
            return Err(Error::MontoInvalido);
        }
        token::Client::new(&env, &token).transfer(&env.current_contract_address(), &a, &monto);
        Pagado { token, a, monto }.publish(&env);
        Ok(())
    }

    /// El dueño actual del agente hace que la cuenta llame a otro contrato, como si la
    /// cuenta fuera quien firma (por ejemplo, contratar a un equipo o dar feedback).
    pub fn ejecutar(env: Env, contrato: Address, funcion: Symbol, args: Vec<Val>) -> Val {
        Self::dueno(env.clone()).require_auth();
        let r: Val = env.invoke_contract(&contrato, &funcion, args);
        Ejecutado { contrato, funcion }.publish(&env);
        r
    }

    /// El dueño hace que la cuenta llame a varios contratos en una sola transacción
    /// (todo o nada). Regresa lo que regresó cada llamada.
    pub fn ejecutar_lote(env: Env, llamadas: Vec<(Address, Symbol, Vec<Val>)>) -> Vec<Val> {
        Self::dueno(env.clone()).require_auth();
        let mut res = Vec::new(&env);
        for (contrato, funcion, args) in llamadas.iter() {
            let r: Val = env.invoke_contract(&contrato, &funcion, args);
            Ejecutado { contrato, funcion }.publish(&env);
            res.push_back(r);
        }
        res
    }

    /// El dueño paga a varios a la vez desde la cuenta del agente (por ejemplo, a los
    /// humanos que ayudaron en un reto). Regresa el total pagado.
    pub fn pagar_lote(env: Env, token: Address, pagos: Vec<(Address, i128)>) -> Result<i128, Error> {
        Self::dueno(env.clone()).require_auth();
        if pagos.is_empty() {
            return Err(Error::MontoInvalido);
        }
        let mut total: i128 = 0;
        for (_, m) in pagos.iter() {
            if m <= 0 {
                return Err(Error::MontoInvalido);
            }
            total += m;
        }
        let t = token::Client::new(&env, &token);
        for (a, m) in pagos.iter() {
            t.transfer(&env.current_contract_address(), &a, &m);
        }
        PagoEnLote { token, pagos: pagos.len(), total }.publish(&env);
        Ok(total)
    }

    /// El dueño deja que `gastador` (por ejemplo, un servicio que el agente usa) cobre
    /// hasta `monto` de la cuenta antes del ledger `hasta_ledger`.
    pub fn aprobar_gasto(env: Env, token: Address, gastador: Address, monto: i128, hasta_ledger: u32) -> Result<(), Error> {
        Self::dueno(env.clone()).require_auth();
        if monto < 0 {
            return Err(Error::MontoInvalido);
        }
        token::Client::new(&env, &token).approve(&env.current_contract_address(), &gastador, &monto, &hasta_ledger);
        GastoAprobado { token, gastador, monto, hasta_ledger }.publish(&env);
        Ok(())
    }

    /// El dueño vacía la cuenta del agente hacia `a`. Regresa lo retirado.
    pub fn retirar_todo(env: Env, token: Address, a: Address) -> Result<i128, Error> {
        Self::dueno(env.clone()).require_auth();
        let t = token::Client::new(&env, &token);
        let monto = t.balance(&env.current_contract_address());
        if monto <= 0 {
            return Err(Error::MontoInvalido);
        }
        t.transfer(&env.current_contract_address(), &a, &monto);
        Retirado { token, a, monto }.publish(&env);
        Ok(monto)
    }

    /// El dueño cancela un permiso de gasto antes de que venza (deja la aprobación en cero).
    pub fn revocar_gasto(env: Env, token: Address, gastador: Address) {
        Self::dueno(env.clone()).require_auth();
        let ledger = env.ledger().sequence();
        token::Client::new(&env, &token).approve(&env.current_contract_address(), &gastador, &0, &ledger);
        GastoRevocado { token, gastador }.publish(&env);
    }

    /// El dueño le da al agente una llave propia (su runtime) para pagar solo, sin que el
    /// dueño firme cada pago, pero nunca más de `tope_por_pago` por pago y solo en `token`.
    /// Llamarla otra vez rota la llave o cambia el tope.
    pub fn fijar_operador(env: Env, llave: Address, token: Address, tope_por_pago: i128) -> Result<(), Error> {
        Self::dueno(env.clone()).require_auth();
        if tope_por_pago <= 0 {
            return Err(Error::MontoInvalido);
        }
        let op = Operador { llave: llave.clone(), token: token.clone(), tope_por_pago };
        env.storage().instance().set(&Clave::Operador, &op);
        OperadorFijado { llave, token, tope_por_pago }.publish(&env);
        Ok(())
    }

    /// El dueño le quita al agente su llave de operador (por ejemplo, si se filtró).
    pub fn quitar_operador(env: Env) -> Result<(), Error> {
        Self::dueno(env.clone()).require_auth();
        let op: Operador = env.storage().instance().get(&Clave::Operador).ok_or(Error::SinOperador)?;
        env.storage().instance().remove(&Clave::Operador);
        OperadorQuitado { llave: op.llave }.publish(&env);
        Ok(())
    }

    /// El agente paga solo con su llave de operador, dentro del tope que fijó el dueño.
    pub fn pago_operador(env: Env, a: Address, monto: i128) -> Result<(), Error> {
        let op: Operador = env.storage().instance().get(&Clave::Operador).ok_or(Error::SinOperador)?;
        op.llave.require_auth();
        if Self::congelada(env.clone()) {
            return Err(Error::Congelada);
        }
        if monto <= 0 {
            return Err(Error::MontoInvalido);
        }
        if monto > op.tope_por_pago {
            return Err(Error::ExcedeTope);
        }
        if let Some(tope) = Self::tope_diario(env.clone()) {
            let dia = env.ledger().timestamp() / 86_400;
            let gastado: i128 = env.storage().temporary().get(&Clave::GastadoHoy(dia)).unwrap_or(0);
            if gastado + monto > tope {
                return Err(Error::ExcedeTopeDiario);
            }
            env.storage().temporary().set(&Clave::GastadoHoy(dia), &(gastado + monto));
            env.storage().temporary().extend_ttl(&Clave::GastadoHoy(dia), 17_280, 34_560);
        }
        token::Client::new(&env, &op.token).transfer(&env.current_contract_address(), &a, &monto);
        PagoDelOperador { llave: op.llave, a, monto }.publish(&env);
        Ok(())
    }

    /// El dueño congela la cuenta: el agente deja de pagar solo con su llave de operador
    /// (por ejemplo, si su runtime se comporta raro), sin perder su configuración. El dueño
    /// sigue pudiendo pagar y retirar.
    pub fn congelar_cuenta(env: Env) -> Result<(), Error> {
        let dueno = Self::dueno(env.clone());
        dueno.require_auth();
        if Self::congelada(env.clone()) {
            return Err(Error::Congelada);
        }
        env.storage().instance().set(&Clave::Congelada, &true);
        CuentaCongelada { por: dueno }.publish(&env);
        Ok(())
    }

    /// El dueño descongela la cuenta: el operador vuelve a pagar dentro de su tope.
    pub fn descongelar_cuenta(env: Env) -> Result<(), Error> {
        let dueno = Self::dueno(env.clone());
        dueno.require_auth();
        if !Self::congelada(env.clone()) {
            return Err(Error::NoCongelada);
        }
        env.storage().instance().remove(&Clave::Congelada);
        CuentaDescongelada { por: dueno }.publish(&env);
        Ok(())
    }

    /// El dueño fija cuánto puede gastar el operador en total por día (UTC), además
    /// del tope por pago: muchos pagos chicos ya no vacían la cuenta.
    pub fn fijar_tope_diario(env: Env, tope: i128) -> Result<(), Error> {
        let dueno = Self::dueno(env.clone());
        dueno.require_auth();
        if tope <= 0 {
            return Err(Error::MontoInvalido);
        }
        env.storage().instance().set(&Clave::TopeDiario, &tope);
        TopeDiarioFijado { por: dueno, tope }.publish(&env);
        Ok(())
    }

    /// El dueño quita el tope diario; queda solo el tope por pago.
    pub fn quitar_tope_diario(env: Env) -> Result<(), Error> {
        let dueno = Self::dueno(env.clone());
        dueno.require_auth();
        if Self::tope_diario(env.clone()).is_none() {
            return Err(Error::SinTopeDiario);
        }
        env.storage().instance().remove(&Clave::TopeDiario);
        TopeDiarioQuitado { por: dueno }.publish(&env);
        Ok(())
    }

    pub fn tope_diario(env: Env) -> Option<i128> {
        env.storage().instance().get(&Clave::TopeDiario)
    }

    pub fn congelada(env: Env) -> bool {
        env.storage().instance().get(&Clave::Congelada).unwrap_or(false)
    }

    /// La llave de operador del agente, si tiene.
    pub fn operador(env: Env) -> Option<Operador> {
        env.storage().instance().get(&Clave::Operador)
    }

    /// Quién controla la cuenta hoy: el dueño del agente.
    pub fn dueno(env: Env) -> Address {
        let agentes: Address = env.storage().instance().get(&Clave::Agentes).unwrap();
        let agente: u32 = env.storage().instance().get(&Clave::Agente).unwrap();
        AgentesClient::new(&env, &agentes).dueno(&agente)
    }

    pub fn agente(env: Env) -> (Address, u32) {
        (
            env.storage().instance().get(&Clave::Agentes).unwrap(),
            env.storage().instance().get(&Clave::Agente).unwrap(),
        )
    }
}

#[cfg(test)]
mod test;
