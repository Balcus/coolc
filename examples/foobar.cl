class FooBar inherits IO {
    n : Int;

    run(): Object {
        n <- in_int();
        let i: Int <- 0 in {
            while i <= n loop {
                let
                    m3: Int <- (i - (i / 3) * 3),
                    m5: Int <- (i - (i / 5) * 5)
                in
                {
                    if m3 = 0 then out_string("Foo") else 0 fi;
                    if m5 = 0 then out_string("Bar") else 0 fi;
                    if (not m3 = 0) then 
                        {
                            if (not m5 = 0) then 
                                out_int(n) 
                            else 
                                0 
                            fi; 
                        } 
                    else 
                        0 
                    fi;
                    out_string("\n");
                    i <- i + 1;
                };
            }pool;
        };
    };
};

class Main {
    foobar: FooBar <- new FooBar;

    main(): Object {
        foobar.run();
    };
};