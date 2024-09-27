#[macro_export]
macro_rules! count_tts {
    () => {0usize};
    ($_head:tt $($tail:tt)*) => {1usize + count_tts!($($tail)*)};
}

#[macro_export]
macro_rules! t_to_maybe_pipelined {
    ($t0: ident) => (MaybePipelinedValue<$t0>);
    ($t0: ident, $($ts: ident),+) => (MaybePipelinedValue<$t0>, t_to_maybe_pipelined!($($ts: ident),+));
}

#[macro_export]
macro_rules! rpc_handler {
    ($fn_id: tt, $fns_map: ident, $t0: ident, $($ts: ident),+) => {
        use ::seq_macro::seq;
        let this = this.clone();
        $fns_map.insert(fn_id, into_handler(move |_conn, pld, slice: Vec<u8>| {
            let this = this.clone();
            async move {
                let args: ($t0: ident, $($ts: ident),+) = if let Some(pld) = pld {
                    let args: (t_to_maybe_pipelined!($t0: ident, $($ts: ident),+)) = rmp_serde::from_slice(&slice).unwrap();

                    (
                        seq!(N in 0..count_tts!($t0: ident, $($ts: ident),+) {
                            args.N.resolve(&pld).await.unwrap(),
                        })
                    )
                } else {
                    rmp_serde::from_slice(&slice).unwrap()
                };

                let out = this.append_string(
                    seq!(N in 0..count_tts!($t0: ident, $($ts: ident),+) {
                        args.N,
                    })
                ).shared().await;
                rmp_serde::to_vec(&out).unwrap()
            }
        }));
    }
}
