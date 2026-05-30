use zcash_primitives::transaction::Transaction;
use zcash_protocol::consensus::{BlockHeight, BranchId};
use zcash_protocol::local_consensus::LocalNetwork;

#[test]
fn parse_shield_tx() {
    let hex = "0600008077777777d80a197700000000bf7e000000000000000000000100d65b378f825ec358c57827a89251980910dab992ab63778dcaf551784d7a72000000006a47304402203074c91bf7e6b73c571b0f0dbec5b0d82e9b13924506364157dda0fc8b0f34ff0220740e6280e555a1734927b032381247aae9d8e26beda12c0e7822f5c380ec37d40121034f690ecfe7189c4b63ac68c0ae1e8c3a7c982841ef8d61aeb71fe52b04066351ffffffff018824a012000000001976a9146415f3e32aed103374add25928d655ee2033976a88ac01000002cc224bf3be53dcf09cc0b3c179b3efc9b800524ac1fd74e29f6379920d22a828f67c3bca240ae7f14c285c2f135b6459a49921db92043a755efc3ac98aa84c1ae3510b30da5167c9490b82c5fdca78bade47edb0506a061d5d8c6819504632e0fcd933b5a7b55cbe8aa1b6cddbfb7f558f44271efb8df25cbee5f351b0ae55d0af419c5916bdfe2e89480351ebe2de72ad2105fedea5b0e2ce450cf130b3e710b10f6f1de179e38c7812b61c25d2842c705d4cb2ffd90980148b37cac0ac55fc6c9dd6c49fffd927b790da80a4f015926bba07275f208877d011ed07e9a1dabdc8ac308fbfca0904de038a57470575556d46a74e70970324920ec86c25d845dce76d90c4a526645e15388df5a44f117677b3eafb32b9800a66a33bbdeaf8aa49084e1d96cddbf8d5a7cf2c397043e56ecc4a468c80806256fd81d7da57633b4ce9dcf08b0d7620bed5222e3082a1676c0ed2ac43ba0896a737dda04480cd19849ccc450ff799475de4fccd17aaa6a4712d97e13282e7ac86195ab15a0ca469c5c65268d7df4387bc2ebb63d494c13a34b122049e8ca109338dc63c7b6026b03504fd540755a21cae15a2f19487727ed3a44df88ca3b78ddfdf67a70d51fd67816b7f70ff31e24bbbfc3e6719214acc2bc3b6e4ed3ba15d84c399941b8d6cb8813e08ee3e27c1a4014ed9521524f3437a2c16594d985b777370bacc789a519a00aefa02570f7d4d9d7c61982959e8980c3aa1137303c470a04f32890e142cb464410353d46caab37ecb2eab62826fc14d43bf4f4ed428d6cd80fea2ee3832ef7d4aba7755bd41e5f9474e10d044ece2ef9ef4f22f571b42b06f208951036250a05bc8da5af110dd78df3680e29bab111450288637924fb0912c40ee7f0d014d34da61ac2bb462e0abcb4174f092dc04eee449e911fd7b4ada180b4d1da03b3a1af102e47dc35fb51a7cf922c1d5e7ac9205e3009f347bc46eed7c4b75e28383117e3246bd18d9a79158ef5ba86c590f985c5c09de2ddf76c0d2fe08d88ced59fe99b6acb0f5fd114651047f19e0cd6a9d434de8fdc92ea636c3f03723809266e91b95a384caef1de97a2e604dda188c3744352937fc2c555bd4700479c8cac26dc1f36bdfe320bbb9c6e21443d81aff305cbe01597c834bc14d85ad1f939398eed115032af23be56a43203fcb7092db96424cd0bd3d7e2c8652b7d116448fb6e62d8d934bbbc8f0e164500643752fda6258e01948c0ffb396627f7c6b29fc9e889258007fa5f5150ba925f058e3d714364bfd54f296c8a4093899ab66ccde5aa6d58399705c04b0faa7cc20c5c6e6b2995f9da2767b5ee266e74968dfb0ee7bd8ec33bdd839c09695f366aa540e85564f876c2991ac4074d4e9b3657b80815dcd71ffe049778f87c7b90414c3c8d7e40013100a5f15d14aa4c105bf46f7577b207c19ec1d96cf298d08118d821dd74cbd96dfdd1f25d11d4a9e58d76eec51ad2c82d5c0ff469ca5eeaad66c9ee5de1e30c5b901459b80005c7bcd8afb1f62e8ae4d991a888eb173815656a132acba67c2660af65d42212675876f74a68fbb001ff4394acf3b25b443839563e0b88db02f1ba565b003f5b9625ca649379cfd7318f15335eb7ba9a0540ccc89cd422d1ccf333681a5aefeaa9e3c2075afc2ff907db1209def5d46f88d77944138b427531fc2e20b9f69f6aa91a77c2aff7cc0fa6a90abdba4afc738eec143f1c335c6b5f6890d1ec7ad6e536f3fba5209ef0d9d16435294e95ff5f872feb23d0e41a006d7ef1ec9aecdd6228da737c1fad7de994a71769eeeb40212b35544fad6567d1988273f86d3b8b72ea56cab56636d07a5126de71fad1c68b3f0c76870e6bdab5c9a65dfcb430f20bf77cf807538ded9ac3cbe98ec99e92b52408920a22cf4bdf2ec0c7c683071d8ed1475be948faf786d1b2b92105d7928c03e685f4b21a13fde061259de2c08119c0316310ac0a807a4429eebd448846f61ccc1417e3000626bc59ac481d357267ccdb8ee79ee8a58efbde2cbbe5e5439570ea47f39f260f53956773adca0cb4a089c928905e4f656ebbf1bb350f6047bb9bbc37d353c20a9e55b5b4fc91e77d2e39d94e1a1277ee1e45f66b9e89039d004f0e0a05fedffffffffa02081e9a1c5e1f0588f9a6abfab0f1019eb9ff0b4ce60b78c53add7623aa4679eeeea61d8ad404d37fbd1db856158c8a4fca1860d571ccc05cdc3dc6c4da97d26da0ced5ee031199001f2cc6f8b257410deff74cdce88bfc89941ae133582ae0bd4567af990396cb17a0f1d0450eaf8676124f503f7391ec85b924b0f7a4b8956714be8ea22920b2080257aebd5e2488815e8197f9667bb6961938cd2d1522d135398e260ae5e5c15b245e9f3b1753c745100c14ca709f6d366c1b95390c05dad725bf4b98c33624f888f0b14126bda827212b9d1d0c6067d7792313989e73faa21986cfd3e7dae13967321202bae7d923ca695064d099a981da526b208fe76bdc448e28db6d96afb3f3a08d0e104436ca2e11a908274af3c8452320be2031607d3fc8d7081242a49dc58589d91032aa2cee12ae86d343354bf72fec873a226a3828e1a939a3bd88783612e45726ba6a0db1a497046683de42d72c6771719e2cc34e690d6342556b41db8a99f16fe4a38316be3f9abef2de89fe449a4b3693101005911ab83fa8582b44ad48d3490d0e4387b3c5c85c6c7b616393a65d4f54900f03f26969d31996c12772b3eb25857f2ee89c151f34f69d3d64dde5858e5de660e000000";

    let data = hex::decode(hex).unwrap();
    println!("Total bytes: {}", data.len());

    let network = LocalNetwork {
        overwinter: Some(BlockHeight::from_u32(1)),
        sapling: Some(BlockHeight::from_u32(1)),
        blossom: Some(BlockHeight::from_u32(1)),
        heartwood: Some(BlockHeight::from_u32(1)),
        canopy: Some(BlockHeight::from_u32(1)),
        nu5: Some(BlockHeight::from_u32(1)),
        nu6: Some(BlockHeight::from_u32(1)),
        nu6_1: Some(BlockHeight::from_u32(1)),
        nu7: Some(BlockHeight::from_u32(1)),
    };

    let height = BlockHeight::from_u32(152);
    let branch_id = BranchId::for_height(&network, height);
    println!("branch_id for height 152: {:?}", branch_id);

    let tx = Transaction::read(&mut &data[..], branch_id).unwrap();
    
    println!("\n=== Transaction ===");
    println!("version: {:?}", tx.version());
    println!("consensus_branch_id: {:?}", tx.consensus_branch_id());
    println!("lock_time: {}", tx.lock_time());
    println!("expiry_height: {}", u32::from(tx.expiry_height()));
    
    println!("\n=== Transparent Bundle ===");
    if let Some(tb) = tx.transparent_bundle() {
        println!("vins: {}", tb.vin.len());
        println!("vouts: {}", tb.vout.len());
        for (i, vout) in tb.vout.iter().enumerate() {
            println!("  vout[{}]: value={}", i, vout.value().into_u64());
        }
    } else {
        println!("NONE");
    }
    
    println!("\n=== Sapling Bundle ===");
    if let Some(sb) = tx.sapling_bundle() {
        println!("spends: {}", sb.shielded_spends().len());
        println!("outputs: {}", sb.shielded_outputs().len());
        for (i, out) in sb.shielded_outputs().iter().enumerate() {
            println!("  output[{}]: enc_ciphertext_len={}, out_ciphertext_len={}", 
                i,
                out.enc_ciphertext().as_ref().len(),
                out.out_ciphertext().len(),
            );
        }
        println!("value_balance: {:?}", sb.value_balance());
    } else {
        println!("NONE");
    }
    
    println!("\n=== Orchard Bundle ===");
    if let Some(ob) = tx.orchard_bundle() {
        println!("has orchard bundle, value_balance: {:?}", ob.value_balance());
    } else {
        println!("NONE");
    }

    println!("\n=== Issue Bundle ===");
    if tx.issue_bundle().is_some() {
        println!("ISSUE BUNDLE PRESENT");
    } else {
        println!("NONE");
    }
    
    // Write back and check size
    let mut written = vec![];
    tx.write(&mut written).unwrap();
    println!("\n=== Roundtrip ===");
    println!("Original size: {}", data.len());
    println!("Roundtrip size: {}", written.len());
    println!("Match: {}", data == written);
    
    if data != written {
        let diff_pos = data.iter().zip(written.iter()).position(|(a, b)| a != b);
        println!("First difference at byte: {:?}", diff_pos);
        if let Some(pos) = diff_pos {
            println!("  Original byte at {}: {:02x}", pos, data[pos]);
            println!("  Written  byte at {}: {:02x}", pos, written[pos]);
        }
    }
}
