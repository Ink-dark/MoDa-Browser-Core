cmake_minimum_required(VERSION 3.20)
project(MoDa-Browser-Core VERSION 0.1.0)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# 启用安全编译选项
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fsanitize=address,undefined")
set(CMAKE_LINKER_FLAGS "${CMAKE_LINKER_FLAGS} -fsanitize=address,undefined")

# 包含目录
include_directories(${CMAKE_SOURCE_DIR}/include)

# 子目录
add_subdirectory(src/security)
add_subdirectory(src/sandbox)
add_subdirectory(src/ipc)
add_subdirectory(src/render)
add_subdirectory(src/network)
add_subdirectory(src/storage)
add_subdirectory(src/platform)
add_subdirectory(examples)
add_subdirectory(tests)

# 安装配置
install(DIRECTORY include/ DESTINATION include/moda)
install(DIRECTORY docs/ DESTINATION share/moda/docs)

# 打包配置
set(CPACK_PACKAGE_NAME "MoDa-Browser-Core")
set(CPACK_PACKAGE_VERSION "${PROJECT_VERSION}")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "MoDa Browser Core - 一个基于最小权限原则的现代模块化浏览器引擎")
set(CPACK_PACKAGE_CONTACT "contact@modabrowser.hallochat.cn")
set(CPACK_RESOURCE_FILE_LICENSE "${CMAKE_SOURCE_DIR}/LICENSE")

include(CPack)