use super::*;
use super::super::Operation;

pub trait Connection<O: Operation> {
    fn send_state(&mut self, state: &State<O>);
}

impl<O: Operation, C: Connection<O>> Connection<O> for Box<C> {
    fn send_state(&mut self, state: &State<O>) {
        (**self).send_state(state)
    }
}

impl<'c, O: Operation, C: Connection<O> + ?Sized + 'c> Connection<O> for &'c mut C {
    fn send_state(&mut self, state: &State<O>) {
        (**self).send_state(state)
    }
}

pub struct Server<O: Operation> {
    history: Vec<State<O>>,
    //connections: Vec<Box<Connection>>,
}

impl<O: Operation> Server<O> {
    pub fn new() -> Self {
        let history = vec![
            State {
                parent: Id(0),
                id: Id(0),
                diff: O::default(),
                content: O::Target::default(),
            },
        ];
        Server {
            history: history,
            //connections: vec![]
        }
    }

    pub fn get_patch(&self, since_id: &Id) -> Result<(Id, O), String> {
        if self.history.len() <= since_id.0 {
            Err("index out of range".into())
        } else {
            let parent_id = Id(self.history.len() - 1);
            let mut op = O::nop(&self.history[since_id.0].content);

            for state in self.history.iter().skip(since_id.0 + 1) {
                op = op.compose(state.diff.clone());
            }

            Ok((parent_id, op))
        }
    }

    pub fn current_state(&self) -> &State<O> {
        self.history.last().unwrap()
    }

    pub fn connect<'a>(&'a mut self, mut connection: Box<Connection<O> + 'a>) {
        connection.send_state(self.current_state());
        //self.connections.push(connection);
    }

    pub fn modify(&mut self, parent: Id, operation: O) -> Result<(Id, O), String> {
        let (parent_id, server_op) = self.get_patch(&parent)?;

        let (server_diff, client_diff) = operation.transform(server_op.clone());
        let content_source = self.history[parent.0].content.clone();

        let id = Id(self.history.len());
        self.history.push(State {
            parent: parent_id.clone(),
            id: id.clone(),
            content: server_op
                .compose(server_diff.clone())
                .apply(&content_source),
            diff: server_diff,
        });

        Ok((id, client_diff))
    }
}
