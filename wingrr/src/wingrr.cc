#include "wingrr.h"

#include <uv.h>
#include <node_embedding_api.h>

#include <mutex>
#include <array>
#include <vector>
#include <string>
#include <memory>
#include <cstring>
#include <cassert>

#include "engines/lua/libwrr-lua.hh"
#include "engines/rb/libwrr-ruby.hh"
#include "engines/py/libwrr-python.hh"
#include "engines/java/libwrr-java.hh"
#include "engines/go/libwrr-go.hh"
#include "engines/cs/libwrr-csharp.hh"

namespace
{
  std::string _get_runtime_path()
  {
    size_t buflen = 4096;
    char buf[buflen]{0};
    uv_os_getenv("WINGRR_ROOT", buf, &buflen);
    if (!buf || buflen == 0 || std::string(buf) == "" && uv_cwd(buf, &buflen) == UV_ENOBUFS)
      return ".";
    else
      return std::string(buf);
  }

  static const std::string RUNTIME_PATH{_get_runtime_path()};

  std::string _resolve(const std::string &rel)
  {
    return RUNTIME_PATH + "/" + rel;
  }
}

extern "C"
{
  struct wingrr_context_t_
  {
    std::mutex mutex;
    const char *program;
    const char *workdir;
    wingrr_engine_type_t type;
  };
  wingrr_context_t *wingrr_prep(wingrr_engine_type_t const type)
  {
    const auto instance = new wingrr_context_t_();
    instance->type = type;
    return instance;
  }
  void wingrr_set_program(wingrr_context_t *const instance, const char *const program)
  {
    assert(instance);
    std::lock_guard<std::mutex> lock(instance->mutex);

    if (instance->program == program)
      return;
    instance->program = program;
  }
  void wingrr_set_workdir(wingrr_context_t *const instance, const char *const context)
  {
    assert(instance);
    std::lock_guard<std::mutex> lock(instance->mutex);

    if (instance->workdir == context)
      return;
    instance->workdir = context;
  }
  int wingrr_exec(wingrr_context_t *const instance)
  {
    int ret = 0;
    assert(instance);
    std::lock_guard<std::mutex> lock(instance->mutex);

    assert(instance->program);
    assert(instance->workdir);

    if (instance->type == WINGRR_ENGINE_JAVASCRIPT ||
        instance->type == WINGRR_ENGINE_TYPESCRIPT)
    {
      std::vector<const char *> argv({"wingrr",
                                      "--experimental-modules",
                                      "--experimental-wasi-unstable-preview1",
                                      "--no-global-search-paths",
                                      "--no-experimental-fetch",
                                      "--no-deprecation",
                                      "--no-warnings",
                                      "--no-addons"});

      if (instance->type == WINGRR_ENGINE_TYPESCRIPT)
      {
        argv.push_back("--require");
        argv.push_back("ts-node/register/transpile-only");
      }

      argv.push_back(instance->program);
      argv.push_back(nullptr);

      size_t buflen = 4096;
      char buf[buflen];
      uv_os_getenv("NODE_PATH", buf, &buflen);
      uv_os_setenv("NODE_PATH", instance->workdir);
      ret = node_main(argv.size() - 1, const_cast<char **>(argv.data()));
      uv_os_setenv("NODE_PATH", buf);
    }

    else if (instance->type == WINGRR_ENGINE_CSHARP)
    {
      wrr::CSharpEngine engine(instance->workdir);
      ret = engine.execute(instance->program);
    }

    else if (instance->type == WINGRR_ENGINE_GO)
    {
      wrr::GoEngine engine(instance->workdir);
      ret = engine.execute(instance->program);
    }

    else if (instance->type == WINGRR_ENGINE_JAVA)
    {
      wrr::JavaEngine engine(instance->workdir);
      ret = engine.execute(instance->program);
    }

    else if (instance->type == WINGRR_ENGINE_PYTHON)
    {
      wrr::PythonEngine engine(instance->workdir);
      ret = engine.execute(instance->program);
    }

    else if (instance->type == WINGRR_ENGINE_RUBY)
    {
      wrr::RubyEngine engine(instance->workdir);
      ret = engine.execute(instance->program);
    }

    else if (instance->type == WINGRR_ENGINE_LUA)
    {
      wrr::LuaEngine engine(instance->workdir);
      ret = engine.execute(instance->program);
    }

    return ret;
  }
  void wingrr_free(wingrr_context_t *const instance)
  {
    delete instance;
  }
}
