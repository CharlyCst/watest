# Watest

A small tool to test wasm modules.

All you have to do is to describe the expected behavior of your module inside a yaml file, like that:

```yaml
file: 'mult.wasm'
funs:
  mult:
    args: [f32, f32]
    out: f32
    test:
      with:
        - [2, 3]
        - [3.14, 2]
        - [4.2, -2]
      expect:
        - 6
        - 6.28
        - -8.4
```

Then run `watest spec.yaml`, if any of the functions returned an unexpected result you will get an error.

## Contributing

Watest is absolutely not production ready, I built it to ease testing the output of a small compiler, but if you feel like it could help you too and you want to add a feature, feel free to open an issue or make a PR!

