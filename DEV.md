## Running test code coverage

```
# runs tests with code coverage and produces cobertura.xml output file
cargo tarpaulin -v -o Xml

# convert cobertura.xml into an HTML report
pycobertura show --format html --output coverage.html cobertura.xml
```

## Profiling apps using [Valgrand](http://valgrind.org/)
There are Valgrind tools that can automatically detect many memory management and threading bugs, and profile your programs in detail.