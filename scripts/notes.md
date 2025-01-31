# Decoding the stats

we have to handle:

- `+%`
- `+`
- `-`
- `per` i.e `totems_attack_speed_+%_per_active_totem`

```rust

Plus(f32);
PlusPercentage(f32);
Minus(f32);
MinusPercentage(f32);
Percent(f32);

```
