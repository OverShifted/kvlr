use std::io::{Cursor, Write};

use kvlr::rpc::rpc_manager::RpcManager;

async fn run(source: &str, rpc_manager: &RpcManager) -> Result<(), ()> {
    let paren_pos = source.chars().position(|c| c == '(').ok_or(())?;
    let _fn_name = &source[..paren_pos];
    let ron_args = &source[paren_pos..];

    let mut msgpack_out = Vec::new();
    
    let mut deserializer = ron::Deserializer::from_str(ron_args).map_err(|_| ())?;
    let mut serializer = rmp_serde::Serializer::new(&mut msgpack_out);
    serde_transcode::transcode(&mut deserializer, &mut serializer).unwrap();

    let result = rpc_manager.call_raw(2000, false, false, msgpack_out).await.unwrap();
    let result = result.await.unwrap().unwrap(); // TODO: Handle errors

    let mut ron_out = Vec::new();
    
    let mut deserializer = rmp_serde::Deserializer::from_read_ref(&result);
    let mut serializer = ron::Serializer::new(&mut ron_out, None).unwrap();    
    serde_transcode::transcode(&mut deserializer, &mut serializer).unwrap();

    let ron_out = String::from_utf8(ron_out).unwrap();

    println!("-> {ron_out}");

    Ok(())
}

pub async fn start(rpc_manager: &RpcManager) {
    let mut source = String::new();

    loop {
        print!(" > ");
        std::io::stdout().flush().unwrap();
        source.clear();
        std::io::stdin().read_line(&mut source).unwrap();

        if source.len() == 0 {
            println!("Goodbye!");
            break;
        }

        let _ = run(&source, rpc_manager).await;
    }
}