#ifndef NANOCODE_H
#define NANOCODE_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* ABI v1. Thread-safe; buffers owned by caller, no overlaps. Bytes, not C strings.
 * Return >=0: bytes produced (zero is a valid empty message).
 * -1: invalid argument/data, -2: payload too long, -3: no valid code, -4: output too small, -5: unsafe palette.
 * All pointers must refer to valid buffers of the documented size.
 */
uint8_t *nc_alloc(size_t size); /* Optional, primarily for WASM. 8-byte aligned. */
void nc_free(uint8_t *ptr, size_t size); /* Exactly same pointer and size. */
/* Returns required SVG length, without NUL. Query with NULL,0 first.
 * Writes nothing when capacity is insufficient. */
int32_t nc_svg(const uint8_t *data, size_t len, uint8_t *out, size_t capacity);
/* Square grayscale raster, size 128..4096; capacity >= size*size. */
int32_t nc_render(const uint8_t *data, size_t len, size_t size, uint8_t *out, size_t capacity);
/* Packed grayscale, 32..2048 each dimension. out capacity >=24.
 * corners: optional eight doubles (x,y) of virtual reference points in payload orientation.
 * Camera adapters must remove row/pixel stride before calling. */
int32_t nc_scan(const uint8_t *gray, size_t width, size_t height,
                uint8_t *out, size_t capacity, double *corners);
/* v2 family. id: 0=auto for generation; 1=S(8B/4 rings), 2=M(16B/6),
 * 3=L(24B/8), 4=XL(40B/10). Invalid id=-1, overflow=-2.
 * Legacy nc_svg/nc_render remain v1. nc_scan reads both versions; -4 if out is too small.
 */
int32_t nc_format_for_len(size_t len);
int32_t nc_format_capacity(uint32_t id);
int32_t nc_format_rings(uint32_t id);
/* RGB 0xRRGGBB, opaque. Score = smallest pairwise 8-bit luma separation.
 * flags: 1 invalid RGB, 2 ink separation <35, 4 background separation <35.
 * flags=0: allowed, score>=55: recommended heuristic. Not a camera guarantee.
 */
int32_t nc_palette_score(uint32_t one, uint32_t zero, uint32_t background);
uint32_t nc_palette_flags(uint32_t one, uint32_t zero, uint32_t background);
/* Returns number of suggested backgrounds (possibly 0), best first. Query NULL,0.
 * Writes only if capacity is sufficient; capacity is in uint32 values. */
int32_t nc_backgrounds(uint32_t one, uint32_t zero, uint32_t *out, size_t capacity);
/* Up to 8 secondary inks for a fixed primary/background; score >=55, best first.
 * Empty means change primary/background. Invalid RGB also returns 0.
 * Query NULL,0; insufficient capacity returns required count without writing. */
int32_t nc_secondary_colors(uint32_t one, uint32_t background, uint32_t *out, size_t capacity);
/* SVG length query and no-NUL semantics match nc_svg. -5 rejects unsafe palette. */
int32_t nc_svg_ex(const uint8_t *data, size_t len, uint32_t id,
    uint32_t one, uint32_t zero, uint32_t background, uint8_t *out, size_t capacity);
int32_t nc_render_ex(const uint8_t *data, size_t len, uint32_t id, size_t size,
    uint32_t one, uint32_t zero, uint32_t background, uint8_t *out, size_t capacity);
/* Use capacity >=40 for every format. -4 leaves all output buffers untouched.
 * metadata optional uint32[2]: format id (0=legacy v1), corrected bits. */
int32_t nc_scan_ex(const uint8_t *gray, size_t width, size_t height,
    uint8_t *out, size_t capacity, double *corners, uint32_t *metadata);
#ifdef __cplusplus
}
#endif
#endif
