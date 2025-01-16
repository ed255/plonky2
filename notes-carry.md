CPU with arithmetic of 32 bits

simulate integers of 128 bits

Rust:
```
type T = [u32; 4];

fn add(a: T, b: T) -> T {
    v0, carry = a[0].add_overflow(b[0]);
    v1, carry = a[1].add_overflow(b[1]).add_overflow(carry);
    v2, carry = a[2].add_overflow(b[2]).add_overflow(carry);
    v3, carry = a[3].add_overflow(b[4]).add_overflow(carry);
    [v0, v1, v2, v3]
}
```

x86_64 add and add with carry instructions
```
add dst, src (dst = dst + src % 2^32, if overflow, CF=1)
adc dst, src (dst = dst + src + CF, if overflow, CF=1)
```

a: [r0, r1, r2, r3]
b: [r4, r5, r6, r7]

x86_64 (with carry)
```
add r0, r4
adc r1, r5
adc r2, r6
adc r3, r7
```

4 inst

without carry
```
add r0, r4
lt c, r0, r4

add r1, r5
lt c0, r1, r4
add r1, c
lt c1, r1, r4
or c, c0, c1

add r2, r6
add r2, c
lt c, r2, r6

add r3, r7
add r3, c
```

10 inst / 14 inst
