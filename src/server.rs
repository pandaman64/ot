
use super::*;
use super::util::*;

pub trait Connection {
    fn send_state(&mut self, state: &State);
}

pub struct Server {
    history: Vec<State>,
    //connections: Vec<Box<Connection>>,
}

impl Server {
    pub fn new() -> Self {
        let history = vec![
            State { parent: Id(0), id: Id(0), operation: Operation::new() }
        ]; 
        Server {
            history: history,
            //connections: vec![]
        }
    }

    pub fn current_state(&self) -> &State {
        self.history.last().unwrap()
    }

    pub fn connect<'a>(&'a mut self, mut connection: Box<Connection + 'a>) {
        connection.send_state(self.current_state());
        //self.connections.push(connection);
    }

    pub fn modify(&mut self, parent: Id, operation: Operation) -> Result<(Id, Operation), String> {
        self.history
            .get(parent.0)
            .ok_or_else(|| "invalid parent id".into())
            .map(|server_op| {
                let mut server_op = server_op.operation.clone();
                for state in self.history[(parent.0 + 1)..].iter() {
                    server_op = compose(server_op, state.operation.clone());
                }
                let (client_op, server_op) = transform(operation, server_op);
                let new_parent = self.history.len() - 1;
                let new_id = self.history.len();
                (Id(new_parent), Id(new_id), client_op, server_op)
            })
            .map(|(parent_id, id, client_op, server_op)| {
                self.history.push(State {
                    parent: parent_id,
                    id: id.clone(),
                    operation: client_op
                });
                (id, server_op)
            })
    }
}

