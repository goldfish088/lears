/*

Every PNG is structured as follows:

[ FILE SIGNATURE ][ CHUNK ]+

FILE SIGNATURE:      [ 0x89 0x50 0x4e ex47 0x0d 0x0a 0x1a 0x0a ] (constraint: must exactly match this)
CHUNK: {
    DATA_LENGTH      [ 0x?? 0x?? 0x?? 0x?? ] (constraint: <= 1<<31)
    CHUNK_TYPE       [ 0x?? 0x?? 0x?? 0x?? ] (constraint: each byte either in [0x41, 0x5a] or [0x61, 0x7a])
    CHUNK_DATA       [ 0x??    ...    0x?? ] (constraint: length == DATA_LENGTH)
    CRC              [ 0x?? 0x?? 0x?? 0x?? ] (constraint: checksum via CRC algo on CHUNK_TYPE + CHUNK_DATA)
}

*/
