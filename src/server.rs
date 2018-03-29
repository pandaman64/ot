
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
            State { 
                parent: Id(0),
                id: Id(0),
                diff: Operation::new(),
                content: "".into() 
            }
        ]; 
        Server {
            history: history,
            //connections: vec![]
        }
    }

    pub fn get_patch(&self, since_id: &Id) -> Result<(Id, Operation), String> {
        if self.history.len() <= since_id.0 {
            Err("index out of range".into())
        } else {
            let parent_id = Id(self.history.len() - 1);
            let mut op = {
                let mut op = Operation::new();
                op.retain(self.history[since_id.0].diff.target_len());
                op
            };

            for state in self.history.iter().skip(since_id.0 + 1) {
                op = compose(op, state.diff.clone());
            }

            Ok((parent_id, op))
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
        let (parent_id, server_op) = self.get_patch(&parent)?;

        let (server_diff, client_diff) = transform(operation, server_op.clone());
        let content_source = self.history[parent.0].content.clone(); 

        let id = Id(self.history.len());
        self.history.push(State {
            parent: parent_id.clone(),
            id: id.clone(),
            content: apply(&content_source, &compose(server_op, server_diff.clone())),
            diff: server_diff,
        });

        Ok((id, client_diff))
    }
}

