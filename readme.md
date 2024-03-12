# C-env-CLI



## Usage

### if you want to genarate project with C++

ex:

```
cenv-cli my_cpp_project 
```


### if C lang
ex:
```
cenc-cli my_c_project gcc -c
```

### Command Help
```
$ cenv-cli --help
Usage: cenv-cli [OPTIONS] <PROJECT_NAME> <BUILD_TYPE>

Arguments:
  <PROJECT_NAME>  Name of Project name
  <BUILD_TYPE>    you can choice the Types for Build. Default: CMake [possible values: gcc, gpp, cmake, clang, clangpp]

Options:
  -c, --c        Default: false | If you want to use C lang
  -x, --cpp      Default: true  | If you want to use C++ lang
  -g, --git      Initialization git and add a .gitignore
  -r, --readme   Add a readme.md file
  -h, --help     Print help
  -V, --version  Print version
```
