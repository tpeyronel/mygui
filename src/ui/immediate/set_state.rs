use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
struct SetStatePacket {
    hash: u64,
    bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct SetStateSender(Rc<RefCell<Vec<SetStatePacket>>>);

impl SetStateSender {
    pub fn send(&self, hash: u64, bytes: Vec<u8>) {
        self.0.borrow_mut().push(SetStatePacket { hash, bytes });
    }
}

pub struct SetStateReceiver(Rc<RefCell<Vec<SetStatePacket>>>);

impl SetStateReceiver {
    pub fn drain(&self, mut f: impl FnMut(u64, Vec<u8>)) {
        for SetStatePacket { hash, bytes } in self.0.borrow_mut().drain(..) {
            f(hash, bytes);
        }
    }
}

pub fn set_state_channel() -> (SetStateSender, SetStateReceiver) {
    let channel = Rc::new(RefCell::new(Vec::new()));
    (SetStateSender(Rc::clone(&channel)), SetStateReceiver(channel))
}
