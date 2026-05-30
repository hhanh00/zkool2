// Parse block 311 using librustzcash deserialization
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use zcash_primitives::transaction::Transaction;
use zcash_protocol::consensus::BranchId;

#[test]
fn parse_block_311_with_zcash_primitives() {
    let block_hex = std::fs::read_to_string("tests/block_311.hex").unwrap().trim().to_string();
    let block_bytes = hex::decode(block_hex).unwrap();
    println!("Total block bytes: {}", block_bytes.len());

    let mut cursor = Cursor::new(block_bytes.as_slice());

    // Block header: version(4) + prev_hash(32) + merkle_root(32) + final_sapling_root(32) + time(4) + bits(4) + nonce(32) = 140
    cursor.set_position(140);
    println!("Byte at 140: 0x{:02x}", block_bytes[140]);
    let solution_len = read_compact_size(&mut cursor);
    println!("Solution len: {}", solution_len);
    cursor.set_position(cursor.position() + solution_len as u64);

    // TX count
    let tx_count = read_compact_size(&mut cursor);
    println!("TX count: {}", tx_count);

    for tx_idx in 0..tx_count {
        let tx_start = cursor.position() as usize;
        let mut tx_slice = &block_bytes[tx_start..];

        println!("\n=== TX {} at offset {} ===", tx_idx, tx_start);

        match Transaction::read(&mut tx_slice, BranchId::Nu7) {
            Ok(tx) => {
                let consumed = block_bytes.len() - tx_start - tx_slice.len();
                println!("✅ TX {} parsed successfully!", tx_idx);
                println!("  Consumed: {} bytes", consumed);
                println!("  txid: {}", tx.txid());

                cursor.set_position((tx_start + consumed) as u64);

                if tx_idx == 1 {
                    let tx_data = tx.into_data();
                    println!("\n  --- Shield TX manual parse for byte positions ---");
                    let tx_bytes = &block_bytes[tx_start..tx_start + consumed];

                    // We know from Rust: version=6, version_group_id=0x77777777
                    // Let's manually walk through the binary to get exact positions
                    let mut pos = 0usize;

                    // header (4) + version_group_id (4) + consensus_branch_id (4) + lock_time (4) + expiry_height (4) + zip233 (8)
                    pos += 4 + 4 + 4 + 4 + 4 + 8; // 28
                    println!("  After header fields: pos={}", pos);

                    // tx_in_count
                    let tx_in_count = read_compact_size_at(tx_bytes, &mut pos);
                    println!("  tx_in_count={} at pos_before={}", tx_in_count, pos - 1);
                    for i in 0..tx_in_count {
                        pos += 32 + 4; // prevout
                        let sl = read_compact_size_at(tx_bytes, &mut pos);
                        pos += sl + 4; // script + sequence
                    }
                    println!("  After tx_in: pos={}", pos);

                    // tx_out_count
                    let tx_out_count = read_compact_size_at(tx_bytes, &mut pos);
                    println!("  tx_out_count={}", tx_out_count);
                    for _ in 0..tx_out_count {
                        pos += 8; // value
                        let sl = read_compact_size_at(tx_bytes, &mut pos);
                        pos += sl; // script
                    }
                    println!("  After tx_out: pos={}", pos);

                    // sighash
                    let sighash_start = pos;
                    for _ in 0..tx_in_count {
                        let sl = read_compact_size_at(tx_bytes, &mut pos);
                        pos += sl;
                    }
                    println!("  Sighash: pos={} (consumed {} bytes)", pos, pos - sighash_start);

                    // Sapling
                    let sapling_start = pos;
                    let spend_count = read_compact_size_at(tx_bytes, &mut pos);
                    println!("  nSpends: {} at {}", spend_count, pos-1);
                    for _ in 0..spend_count { pos += 32*4; }
                    let output_count = read_compact_size_at(tx_bytes, &mut pos);
                    println!("  nOutputs: {} at {}", output_count, pos-1);
                    for _ in 0..output_count { pos += 32*3 + 580 + 80; }
                    if spend_count+output_count > 0 { pos += 8; }
                    if spend_count > 0 { pos += 32; }
                    pos += spend_count * 192;
                    for _ in 0..spend_count { let sl = read_compact_size_at(tx_bytes, &mut pos); pos += sl + 64; }
                    pos += output_count * 192;
                    if spend_count+output_count > 0 { let sl = read_compact_size_at(tx_bytes, &mut pos); pos += sl + 64; }
                    println!("  Sapling end: pos={} (consumed {} bytes)", pos, pos - sapling_start);

                    // Orchard
                    let orchard_start = pos;
                    let num_ag = read_compact_size_at(tx_bytes, &mut pos);
                    println!("  nActionGroups: {} at {}", num_ag, pos-1);
                    if num_ag == 1 {
                        let ac = read_compact_size_at(tx_bytes, &mut pos);
                        println!("  nActions: {} at {}", ac, pos-1);
                        println!("  Action start: pos={}", pos);
                        for i in 0..ac {
                            let a_start = pos;
                            pos += 32*5 + 612 + 80;
                            println!("    action[{}]: {} bytes ({} to {})", i, pos - a_start, a_start, pos);
                        }
                        let flags_pos = pos;
                        pos += 1; // flags
                        println!("  flags: pos={}", flags_pos);
                        let anchor_pos = pos;
                        pos += 32; // anchor
                        println!("  anchor: pos={}", anchor_pos);
                        let ag_pos = pos;
                        pos += 4; // nAGExpiryHeight
                        println!("  nAGExpiryHeight: pos={}", ag_pos);
                        let burn_pos = pos;
                        let bc = read_compact_size_at(tx_bytes, &mut pos);
                        println!("  burn count={} at {}", bc, burn_pos);
                        pos += bc * 40;
                        let proofs_pos = pos;
                        let pc = read_compact_size_at(tx_bytes, &mut pos);
                        println!("  proofs: len={} at {} (CompactSize byte at {})", pc, proofs_pos, proofs_pos);
                        pos += pc;
                        println!("  proofs end: pos={}", pos);
                        let sigs_pos = pos;
                        println!("  vSpendAuthSigs start: pos={}", sigs_pos);
                        for i in 0..ac {
                            let sl = read_compact_size_at(tx_bytes, &mut pos);
                            println!("    sig[{}]: CompactSize={} at pos {}", i, sl, pos-1);
                            pos += sl;
                            pos += 64;
                            println!("    sig[{}]: end pos {}", i, pos);
                        }
                        let vb_pos = pos;
                        pos += 8; // valueBalance
                        println!("  valueBalance: pos={}", vb_pos);
                        let bsig_pos = pos;
                        println!("  bindingSig START: pos={}", bsig_pos);
                        let bsl = read_compact_size_at(tx_bytes, &mut pos);
                        println!("  bindingSig CompactSize: {} at {}", bsl, pos-1);
                        pos += bsl; // sighash data
                        pos += 64; // sig
                        println!("  bindingSig END: pos={}", pos);
                    }
                    // Issue bundle
                    let issue_pos = pos;
                    let il = read_compact_size_at(tx_bytes, &mut pos);
                    println!("  issue bundle: issuer_len={} at {}", il, pos-1);
                    if il == 0 {
                        let na = read_compact_size_at(tx_bytes, &mut pos);
                        println!("  issue bundle: nActions={} at {}", na, pos-1);
                    }
                    println!("  issue bundle END: pos={}", pos);
                    let total = pos;
                    println!("\n  === SUMMARY ===");
                    println!("  Total parsed: {} bytes", total);
                    println!("  TX consumed: {} bytes", consumed);
                    println!("  Match: {}", total == consumed);
                }
            }
            Err(e) => {
                println!("❌ TX {} FAILED: {:?}", tx_idx, e);
                println!("  First 100 bytes: {}", hex::encode(&tx_slice[..std::cmp::min(100, tx_slice.len())]));
                panic!("Transaction::read failed for TX {}", tx_idx);
            }
        }
    }

    let remaining = block_bytes.len() as u64 - cursor.position();
    println!("\n✅ All transactions parsed! Remaining bytes: {}", remaining);
}

fn read_compact_size_at(data: &[u8], pos: &mut usize) -> usize {
    let len_byte = data[*pos];
    *pos += 1;
    match len_byte {
        n if n < 253 => n as usize,
        253 => {
            let val = u16::from_le_bytes([data[*pos], data[*pos+1]]) as usize;
            *pos += 2;
            val
        }
        254 => {
            let val = u32::from_le_bytes([data[*pos], data[*pos+1], data[*pos+2], data[*pos+3]]) as usize;
            *pos += 4;
            val
        }
        _ => panic!("bad compact size: {}", len_byte),
    }
}

fn read_compact_size(cursor: &mut Cursor<&[u8]>) -> usize {
    let len_byte = cursor.read_u8().unwrap();
    match len_byte {
        n if n < 253 => n as usize,
        253 => cursor.read_u16::<LittleEndian>().unwrap() as usize,
        254 => cursor.read_u32::<LittleEndian>().unwrap() as usize,
        _ => panic!("bad compact size: {}", len_byte),
    }
}
