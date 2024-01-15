fun fib(n) {
    if(n < 2) {
        return n;
    }

    var a0 = 0;
    var a1 = 1;
    var cnt = 2;
    while(cnt <= n) {
        print cnt - 2;
        var tmp = a0;
        a0 = a1;
        a1 = a1 + tmp;
        cnt = cnt + 1;
    }

    return a1;
}

print now(10, "hogehoge");
var num = 42;
print "fib(" + num + ") = " + fib(num);
print now();
