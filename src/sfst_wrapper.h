#ifndef SFST_WRAPPER_H
#define SFST_WRAPPER_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Opaque handle owning a single loaded transducer. Each handle is independent;
 * dropping one never affects another.
 */
typedef struct SfstHandle SfstHandle;

/**
 * Load a transducer from a file.
 * On success returns a non-NULL handle and sets *err to 0.
 * On failure returns NULL and sets *err to:
 *   1 - filename is null
 *   2 - could not open file
 *   3 - error loading transducer
 * The handle must be released with sfst_cleanup.
 */
SfstHandle *sfst_init(const char *filename, int *err);

/**
 * Release a handle and free its transducer. Passing NULL is a no-op.
 */
void sfst_cleanup(SfstHandle *handle);

/**
 * Analyze a string and return results.
 * result_count will be set to the number of results.
 * Returns array of strings that must be freed with sfst_free_results.
 */
char **sfst_analyse(SfstHandle *handle, const char *input, int *result_count);

/**
 * Generate a string and return results.
 * result_count will be set to the number of results.
 * Returns array of strings that must be freed with sfst_free_results.
 */
char **sfst_generate(SfstHandle *handle, const char *input, int *result_count);

/**
 * Free the results returned by sfst_analyse or sfst_generate.
 */
void sfst_free_results(char **results, int count);

#ifdef __cplusplus
}
#endif

#endif
