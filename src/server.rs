
use super::*;
use super::util::*;

pub trait Connection {
    fn send_state(&mut self, state: &State);
}

pub struct Server {
    history: Vec<State>,
    connections: Vec<Box<Connection>>,
}

impl Server {
    pub fn new() -> Self {
        let history = vec![
            State { parent: Id(0), operation: Operation::new() }
        ]; 
        Server {
            history: history,
            connections: vec![]
        }
    }

    pub fn current_state(&self) -> &State {
        self.history.last().unwrap()
    }

    pub fn connect(&mut self, mut connection: Box<Connection>) {
        connection.send_state(self.current_state());
        self.connections.push(connection);
    }

    pub fn modify(&mut self, state: State) -> Result<Operation, ()> {
        if state.parent >= Id(self.history.len() - 1) {
            return Err(());
        }
        let mut server_op = self.history[state.parent.0].operation.clone();
        for state in self.history[(state.parent.0 + 1)..].iter() {
            server_op = compose(server_op, state.operation.clone());
        }
        let (client_op, server_op) = transform(state.operation, server_op);
        let new_parent = self.history.len() - 1;
        self.history.push(State{
            parent: Id(new_parent),
            operation: client_op
        });
        Ok(server_op)
    }
}

