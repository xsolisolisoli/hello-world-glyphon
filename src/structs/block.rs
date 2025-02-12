struct Block {
    pub is_active: bool,
    pub is_solid: bool,
    pub is_transparent: bool,
    pub block_type: BlockType,
}


enum BlockType {
    BlockType_Default = 0,
    BlockType_Grass = 1,
}

struct Chunk {
    pub blocks: Vec<Block>,
    pub chunk_pos: (i32, i32),
}