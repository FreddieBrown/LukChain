#[derive(Clone, Debug)]
pub struct BlockChain {
    pub chain: Vec<Block>,
    pending_chats: Vec<Chat>,
}

impl BlockChain {
    pub fn new() -> Self {
        Self {
            chain: Vec::new(),
            pending_chats: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    pub chats: Vec<Chat>,
    pub prev_hash: Option<String>,
    pub hash: Option<String>,
    pub nonce: u128,
}

impl Block {
    pub fn new(prev_hash: Option<String>) -> Self {
        Self {
            chats: Vec::new(),
            prev_hash,
            hash: None,
            nonce: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Chat {
    pub data: String,
}

impl Chat {
    pub fn new(data: String) -> Self {
        Self { data }
    }
}
