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

# 只添加存在的子目录
# 注意：当前项目主要是Rust项目，使用Cargo管理，
# CMake配置只用于可选的C++组件

# 安装配置
install(DIRECTORY include/ DESTINATION include/moda)

# 打包配置 - 简化配置，移除不存在的文件引用
set(CPACK_PACKAGE_NAME "MoDa-Browser-Core")
set(CPACK_PACKAGE_VERSION "${PROJECT_VERSION}")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "MoDa Browser Core - 一个基于最小权限原则的现代模块化浏览器引擎")
set(CPACK_PACKAGE_CONTACT "contact@modabrowser.hallochat.cn")

# 只有当LICENSE文件存在时才设置，否则跳过
if(EXISTS "${CMAKE_SOURCE_DIR}/LICENSE")
    set(CPACK_RESOURCE_FILE_LICENSE "${CMAKE_SOURCE_DIR}/LICENSE")
endif()

include(CPack)
