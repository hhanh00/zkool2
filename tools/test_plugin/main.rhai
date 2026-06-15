// DKG DK00 Memo Parser for zkool
//
// Parses FROST DKG Round 0 messages.
//
// 511-byte memo payload format:
//   bytes 0-3:   prefix "DK00" (0x44 0x4b 0x30 0x30)
//   byte  4:     from_id (u8, participant ID)
//   bytes 5-12:  data length (u64 LE, bincode legacy)
//   bytes 13+:   data bytes (VerifyingKey)

fn get_prefixes() {
    return ["444b3030"];  // "DK00" in hex
}

fn process_memo() {
    let from_id = memo::read_u8(4);
    let data_len = memo::read_u64_le(5);

    let headers = ["Field", "Value"];
    let rows = [
        [cell_string("Round"), cell_string("DKG Round 0")],
        [cell_string("Prefix"), cell_string("DK00")],
        [cell_string("From (Participant ID)"), cell_number(from_id)],
        [cell_string("Data Size (bytes)"), cell_number(data_len)],
    ];

    return [section("DKG Message", headers, rows)];
}
