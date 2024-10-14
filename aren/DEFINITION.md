# Types
1. Fixed-size integers: (u)?int{8, 16, 32, 64}
2. Dynamically-sized integer (positive only): dynint
3. Fixed-sized array of n Ts: T[n]
4. Unknown-sized array of Ts: T[]
5. Bytes: bytes (alias for uint8[])
6. Atom: [dynint type id] + [data]
   An atom is like an "object" in Java. Is starts with an dynint
   type identifier which allows the decoder to lookup its size
   and potentially cut it as a slice for later use. [data] is
   usually a list of other objects (integers, arrays, other atoms)
   packed next to each other.
