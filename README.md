# 🔗 glue

[![GitHub license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/mikesposito/glue/blob/main/LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/mikesposito/glue/blob/main/CONTRIBUTING.md)

Make requests, select JSON responses, nest them in other requests: A magnificent syntax for blazingly fast cli HTTP calls, **made for humans**.

## Table of Contents

- [Install & Update](#install--update)
- [Usage](#usage)  
  - [Simple request](#simple-request)
  - [JSON Result selector](#json-result-selector)
  - [Body attributes](#body-attributes)
  - [Nested requests](#nested-requests)
  - [Run file](#run-file)
- [Contributing](#contributing)
  - [Code of conduct](#code-of-conduct)
  - [Contributing Guide](#contributing-guide)
  - [Good First Issues](#good-first-issues)
- [License](#license)

## Install & Update

At the moment, you can install or update glue for your system by building it from source. It has been done quite easy by the script `install.sh`.

All build dependencies (including Rust) will be deleted right after the installation automatically if Rust wasn't already on your system.

1. Clone the repo
```bash
git clone https://github.com/mikesposito/glue
```

2. Go to glue root directory
```bash
cd glue
```

3. Add execute permission and run install.sh
```bash
chmod +x ./install.sh && ./install.sh
```

## Usage

### Simple request

To execute a request chain with glue you can simply pass it this way:

```bash
glue <REQUEST>
```

The simplest request you can do with glue is using simply the method and the url:

```bash
glue "get https://dog.ceo/api/breeds/list/all"
```

### JSON result selector

If the response is of type JSON, you can add a jsonpath selector to the request with the char `^`. Glue will only return the desired value from the response. This applies also for [Nested requests](#nested-requests).

```bash
glue "get https://dog.ceo/api/breeds/list/all^$.message.terrier"

# OUTPUT:
# > [GET] https://dog.ceo/api/breeds/list/all
# 
# 
# [
#   [
#     "american",
#     "australian",
#     "bedlington",
#     "border",
#     "cairn",
#     "dandie",
#     "fox",
#     "irish",
#     "kerryblue",
#     "lakeland",
#     "norfolk",
#     "norwich",
#     "patterdale",
#     "russell",
#     "scottish",
#     "sealyham",
#     "silky",
#     "tibetan",
#     "toy",
#     "welsh",
#     "westhighland",
#     "wheaten",
#     "yorkshire"
#   ]
# ]
```

### Body attributes

You can use the char `~` to add body attributes to the request:

```bash
glue "post https://example.com/user/add~username=admin"
# or
glue "post https://example.com/user/add ~username=admin"

# glue will send a body of type JSON 
# with a key "username" with value "admin"
```

### Nested requests

One of the most useful features of glue is the request nesting. 

You can reuse response values (total or partial) from a request to build another request. 

Glue supports infinite nesting and will build a dependency tree, divide it in layers and execute each layer on parallel for the maximum time optimization.

You can use request nesting delimiting the desired nested request with `{}`:

```bash
glue "get api.com/users/{ get api.com/me^$.user.id }/"

# glue will execute this two requests:

# 1. api.com/me - and will select user.id from the response (eg. 12345)

# 2. api.com/users/12345/
```

### Run file

You can also create a file with request to run, and pass the file path to glue with flag `-f` to execute it. You can try with one of the sample requests in `examples` folder:

```bash
glue -f examples/sample-request.glue
```

## Contributing

The main purpose of this repository is to continue evolving glue core, making it faster and easier to use. Development of glue happens in the open on GitHub, and we are grateful to the community for contributing bugfixes and improvements. Read below to learn how you can take part in improving glue.

### [Code of Conduct](CODE_OF_CONDUCT.md)

glue has adopted a Code of Conduct that we expect project participants to adhere to. Please read [the full text](CODE_OF_CONDUCT.md) so that you can understand what actions will and will not be tolerated.

### [Contributing Guide](CONTRIBUTING.md)

Read our [contributing guide](CONTRIBUTING.md) to learn about our development process, how to propose bugfixes and improvements, and how to build and test your changes to glue.

### Good First Issues

To help you get your feet wet and get you familiar with our contribution process, we have a list of [good first issues](https://github.com/mikesposito/glue/labels/good%20first%20issue) that contain bugs which have a relatively limited scope. This is a great place to get started.

## License

glue is [MIT licensed](./LICENSE).