class Main inherits IO {
    s: Int <- 0;

    main(): Object {
        let
            n: Int <- in_int(),
            i: Int <- 0
        in {
            while i <= n loop {
                s <- s + i;
                i < i + 1;
            } pool;
        };
        out_int(s);
    };
};
