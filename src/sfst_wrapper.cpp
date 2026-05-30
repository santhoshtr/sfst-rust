#include "sfst_wrapper.h"
#include "sfst/src/fst.h"
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

using namespace SFST;

// Each handle owns exactly one transducer. There is no shared global state, so
// distinct handles are fully independent and a single loaded transducer is
// read-only after construction (the binary-file constructor freezes the node
// index), making concurrent queries on one handle race-free.
struct SfstHandle {
  Transducer *transducer;
};

extern "C" {

SfstHandle *sfst_init(const char *filename, int *err) {
  if (filename == nullptr) {
    if (err != nullptr) {
      *err = 1;
    }
    return nullptr;
  }

  FILE *transducer_file = fopen(filename, "rb");
  if (transducer_file == nullptr) {
    if (err != nullptr) {
      *err = 2;
    }
    return nullptr;
  }

  try {
    SfstHandle *handle = new SfstHandle;
    handle->transducer = new Transducer(transducer_file);
    fclose(transducer_file);
    if (err != nullptr) {
      *err = 0;
    }
    return handle;
  } catch (...) {
    fclose(transducer_file);
    if (err != nullptr) {
      *err = 3;
    }
    return nullptr;
  }
}

void sfst_cleanup(SfstHandle *handle) {
  if (handle != nullptr) {
    delete handle->transducer;
    delete handle;
  }
}

char **sfst_analyse(SfstHandle *handle, const char *input, int *result_count) {
  if (handle == nullptr || handle->transducer == nullptr || input == nullptr ||
      result_count == nullptr) {
    if (result_count != nullptr) {
      *result_count = 0;
    }
    return nullptr;
  }

  try {
    std::vector<std::string> results =
        handle->transducer->analyze_string(const_cast<char *>(input), true);
    *result_count = static_cast<int>(results.size());

    if (results.empty()) {
      return nullptr;
    }

    char **c_results =
        static_cast<char **>(malloc(results.size() * sizeof(char *)));
    if (c_results == nullptr) {
      *result_count = 0;
      return nullptr;
    }

    for (size_t i = 0; i < results.size(); i++) {
      size_t len = results[i].length() + 1;
      c_results[i] = static_cast<char *>(malloc(len));
      if (c_results[i] == nullptr) {
        // Clean up on allocation failure
        for (size_t j = 0; j < i; j++) {
          free(c_results[j]);
        }
        free(c_results);
        *result_count = 0;
        return nullptr;
      }
      strcpy(c_results[i], results[i].c_str());
    }

    return c_results;
  } catch (...) {
    *result_count = 0;
    return nullptr;
  }
}

char **sfst_generate(SfstHandle *handle, const char *input, int *result_count) {
  if (handle == nullptr || handle->transducer == nullptr || input == nullptr ||
      result_count == nullptr) {
    if (result_count != nullptr) {
      *result_count = 0;
    }
    return nullptr;
  }

  try {
    std::vector<std::string> results =
        handle->transducer->generate_string(const_cast<char *>(input), true);
    *result_count = static_cast<int>(results.size());

    if (results.empty()) {
      return nullptr;
    }

    char **c_results =
        static_cast<char **>(malloc(results.size() * sizeof(char *)));
    if (c_results == nullptr) {
      *result_count = 0;
      return nullptr;
    }

    for (size_t i = 0; i < results.size(); i++) {
      size_t len = results[i].length() + 1;
      c_results[i] = static_cast<char *>(malloc(len));
      if (c_results[i] == nullptr) {
        // Clean up on allocation failure
        for (size_t j = 0; j < i; j++) {
          free(c_results[j]);
        }
        free(c_results);
        *result_count = 0;
        return nullptr;
      }
      strcpy(c_results[i], results[i].c_str());
    }

    return c_results;
  } catch (...) {
    *result_count = 0;
    return nullptr;
  }
}

void sfst_free_results(char **results, int count) {
  if (results == nullptr) {
    return;
  }

  for (int i = 0; i < count; i++) {
    if (results[i] != nullptr) {
      free(results[i]);
    }
  }
  free(results);
}
}
