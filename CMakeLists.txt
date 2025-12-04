# CMakeLists.txt for Elbo Studio Core Interfaces
# Copyright (C) 2025 Nicholas Wierzbowski / Elbo Studio
# Defines the core interface library for all Elbo Studio components.

if(NOT TARGET core)
  add_library(core INTERFACE)
  target_include_directories(core INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
  if(MSVC)
    target_compile_definitions(core INTERFACE BOOST_ALL_NO_LIB BOOST_INTERPROCESS_NO_LIB)
  endif()
endif()