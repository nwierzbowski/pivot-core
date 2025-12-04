# Elbo Studio Core Interfaces

This repository contains the core shared components for the Pivot project, including header files with enums used for inter-process communication (IPC).

## Overview

The core module provides essential definitions that are shared across different parts of the Pivot ecosystem. Currently, it includes:

- **classification.h**: Defines enums for surface classification, such as `SurfaceType`, which categorizes surfaces into Ground, Wall, Ceiling, or Unknown.

These enums are utilized by the proprietary Pivot Engine to facilitate communication via IPC with other processes, particularly the Pivot Bridge for Blender integration.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Usage

To use the enums in your C++ project, simply include the header file:

```cpp
#include "classification.h"
```

Ensure that the header is in your include path during compilation.
