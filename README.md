# Some basic examples to try `tracing_opentelemetry` with

Based mostly on <https://github.com/MaterializeInc/materialize/blob/feb46fec9c4174156624521a1f209ae7b8723569/src/ore/src/tracing.rs>


## Otel per-layer-filter issues:
Note these sleep after being run so the traces are flushed to tempo

### Base case (all INFO)
```
cargo run -- --weird-case --name all_info
```
![image](https://github.com/guswynn/otel-examples/assets/5404303/7a812505-4479-4e5c-9c11-c15267d0ef07)


### Debug case (all DEBUG)
```
cargo run -- --weird-case --name all_info -f -o
````
![image](https://github.com/guswynn/otel-examples/assets/5404303/11923605-94c4-413c-8c6c-9660e0394443)

### Broken case (fmt layer DEBUG)
```
cargo run -- --weird-case --name all_info -f
```

(they are disconnected):
![image](https://github.com/guswynn/otel-examples/assets/5404303/2ed9c711-5eb2-4a54-91c8-64713ffcbb50)
![image](https://github.com/guswynn/otel-examples/assets/5404303/3587d023-428e-440d-ba3d-31d8902a6e98)

