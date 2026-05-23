// 渲染桥.c — 玄铁渲染库 C 桥接层 v1.0
// 将 XuanTie 的 i64 标记指针转换为 raylib 原生 C 类型并调用
// 编译时与 xt_runtime.c + libraylib.a 一起链接
//
// 标记方案 (from xt_runtime.h):
//   整数: tagged = (raw << 1) | 1   →  untag: raw = (int64_t)(tagged >> 1)
//   布尔: true = 0x4, false = 0x2   →  untag: tagged == 0x4
//   浮点: 存储在 XTFloat 对象中
//   字符串: XTString 对象指针
//   对象: 原始指针 (低位为 0，因为对齐)

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

// === XuanTie 运行时类型（精简版，避免依赖完整 xt_runtime.h） ===
// 标记值操作
#define XT_TAG_INT     0x1ULL
#define XT_FROM_INT(i) (((uintptr_t)(i) << 1) | XT_TAG_INT)
#define XT_TO_INT(v)   ((int64_t)((intptr_t)(v) >> 1))

#define XT_TRUE         ((uintptr_t)0x4ULL)
#define XT_FALSE        ((uintptr_t)0x2ULL)
#define XT_FROM_BOOL(b) ((b) ? XT_TRUE : XT_FALSE)
#define XT_TO_BOOL(v)   ((v) == XT_TRUE)

// XTString 结构 (与 xt_runtime.h 对齐)
typedef struct {
    uint32_t magic;
    int32_t  ref_count;
    int32_t  type_id;
    int32_t  length;
    char     data[];
} XTString;

// XTFloat 结构 (与 xt_runtime.h 对齐)
typedef struct {
    uint32_t magic;
    int32_t  ref_count;
    int32_t  type_id;
    double   value;
} XTFloat;

// === 类型转换辅助宏 ===

// 从 XTValue (uintptr_t/i64) 判断类型
// 排除 XT_FALSE(0x2)/XT_TRUE(0x4)/XT_NULL(0x0) 等小常量——合法堆地址 > 4096
#define IS_PTR(v)       (((uintptr_t)(v) & 0x1) == 0 && (v) > 4096)
#define IS_INT(v)       (((uintptr_t)(v) & 0x1) == 1)

// 提取字符串: 如果是对象指针则返回 data，否则返回空串
static const char* xt_get_cstr(uintptr_t v) {
    if (IS_PTR(v) && v != 0) {
        XTString* s = (XTString*)v;
        if (s->type_id == 3) return s->data;
    }
    return "";
}

// 提取浮点数: 如果是 XTFloat 对象则返回 value
static double xt_get_float(uintptr_t v) {
    if (IS_PTR(v) && v != 0) {
        XTFloat* f = (XTFloat*)v;
        if (f->type_id == 2) return f->value;
    }
    return 0.0;
}

// 创建浮点数 XTValue (返回 i64 兼容值)
// 简化实现：将 double 的位模式直接返回
// 调用者通过 外 声明 float 类型来让编译器正确处理
// 注意：这需要编译器支持，目前我们用简化方案
static uintptr_t xt_make_float(double v) {
    // 在栈上创建一个临时 XTFloat 结构并返回其指针
    // 警告：仅用于立即返回值场景
    // 正确做法：分配 XTFloat 对象并注册到 GC
    XTFloat* f = (XTFloat*)calloc(1, sizeof(XTFloat));
    f->magic = 0x58544F42;
    f->ref_count = 0x7FFFFFFF; // IMMORTAL
    f->type_id = 2;
    f->value = v;
    return (uintptr_t)f;
}

// === raylib 头 ===
#include "raylib.h"

// ============================================================
// 窗口管理
// ============================================================

void XT_InitWindow(uintptr_t w, uintptr_t h, uintptr_t title) {
    InitWindow((int)XT_TO_INT(w), (int)XT_TO_INT(h), xt_get_cstr(title));
}

uintptr_t XT_WindowShouldClose(void) {
    return XT_FROM_BOOL(WindowShouldClose());
}

void XT_CloseWindow(void) {
    CloseWindow();
}

uintptr_t XT_IsWindowReady(void) {
    return XT_FROM_BOOL(IsWindowReady());
}

void XT_SetTargetFPS(uintptr_t fps) {
    SetTargetFPS((int)XT_TO_INT(fps));
}

uintptr_t XT_GetFPS(void) {
    return XT_FROM_INT(GetFPS());
}

uintptr_t XT_GetFrameTime(void) {
    return xt_make_float(GetFrameTime());
}

uintptr_t XT_GetScreenWidth(void) {
    return XT_FROM_INT(GetScreenWidth());
}

uintptr_t XT_GetScreenHeight(void) {
    return XT_FROM_INT(GetScreenHeight());
}

void XT_SetWindowTitle(uintptr_t title) {
    SetWindowTitle(xt_get_cstr(title));
}

// ============================================================
// 绘图帧控制
// ============================================================

void XT_BeginDrawing(void) {
    BeginDrawing();
}

void XT_EndDrawing(void) {
    EndDrawing();
}

void XT_ClearBackground(uintptr_t r, uintptr_t g, uintptr_t b, uintptr_t a) {
    Color c = {
        (unsigned char)XT_TO_INT(r),
        (unsigned char)XT_TO_INT(g),
        (unsigned char)XT_TO_INT(b),
        (unsigned char)XT_TO_INT(a)
    };
    ClearBackground(c);
}

// ============================================================
// 形状绘制
// ============================================================

void XT_DrawPixel(uintptr_t x, uintptr_t y, uintptr_t r, uintptr_t g, uintptr_t b, uintptr_t a) {
    Color c = {(unsigned char)XT_TO_INT(r), (unsigned char)XT_TO_INT(g),
               (unsigned char)XT_TO_INT(b), (unsigned char)XT_TO_INT(a)};
    DrawPixel((int)XT_TO_INT(x), (int)XT_TO_INT(y), c);
}

void XT_DrawLine(uintptr_t x1, uintptr_t y1, uintptr_t x2, uintptr_t y2,
                 uintptr_t r, uintptr_t g, uintptr_t b, uintptr_t a) {
    Color c = {(unsigned char)XT_TO_INT(r), (unsigned char)XT_TO_INT(g),
               (unsigned char)XT_TO_INT(b), (unsigned char)XT_TO_INT(a)};
    DrawLine((int)XT_TO_INT(x1), (int)XT_TO_INT(y1),
             (int)XT_TO_INT(x2), (int)XT_TO_INT(y2), c);
}

void XT_DrawRectangle(uintptr_t x, uintptr_t y, uintptr_t w, uintptr_t h,
                      uintptr_t r, uintptr_t g, uintptr_t b, uintptr_t a) {
    Color c = {(unsigned char)XT_TO_INT(r), (unsigned char)XT_TO_INT(g),
               (unsigned char)XT_TO_INT(b), (unsigned char)XT_TO_INT(a)};
    DrawRectangle((int)XT_TO_INT(x), (int)XT_TO_INT(y),
                  (int)XT_TO_INT(w), (int)XT_TO_INT(h), c);
}

void XT_DrawRectangleRec(uintptr_t x, uintptr_t y, uintptr_t w, uintptr_t h,
                         uintptr_t r, uintptr_t g, uintptr_t b, uintptr_t a) {
    Color c = {(unsigned char)XT_TO_INT(r), (unsigned char)XT_TO_INT(g),
               (unsigned char)XT_TO_INT(b), (unsigned char)XT_TO_INT(a)};
    Rectangle rec = {(float)XT_TO_INT(x), (float)XT_TO_INT(y),
                     (float)XT_TO_INT(w), (float)XT_TO_INT(h)};
    DrawRectangleRec(rec, c);
}

void XT_DrawRectangleLines(uintptr_t x, uintptr_t y, uintptr_t w, uintptr_t h,
                           uintptr_t r, uintptr_t g, uintptr_t b, uintptr_t a) {
    Color c = {(unsigned char)XT_TO_INT(r), (unsigned char)XT_TO_INT(g),
               (unsigned char)XT_TO_INT(b), (unsigned char)XT_TO_INT(a)};
    DrawRectangleLines((int)XT_TO_INT(x), (int)XT_TO_INT(y),
                       (int)XT_TO_INT(w), (int)XT_TO_INT(h), c);
}

void XT_DrawCircle(uintptr_t cx, uintptr_t cy, uintptr_t radius,
                   uintptr_t r, uintptr_t g, uintptr_t b, uintptr_t a) {
    Color c = {(unsigned char)XT_TO_INT(r), (unsigned char)XT_TO_INT(g),
               (unsigned char)XT_TO_INT(b), (unsigned char)XT_TO_INT(a)};
    DrawCircle((int)XT_TO_INT(cx), (int)XT_TO_INT(cy), (float)XT_TO_INT(radius), c);
}

void XT_DrawCircleLines(uintptr_t cx, uintptr_t cy, uintptr_t radius,
                        uintptr_t r, uintptr_t g, uintptr_t b, uintptr_t a) {
    Color c = {(unsigned char)XT_TO_INT(r), (unsigned char)XT_TO_INT(g),
               (unsigned char)XT_TO_INT(b), (unsigned char)XT_TO_INT(a)};
    DrawCircleLines((int)XT_TO_INT(cx), (int)XT_TO_INT(cy), (float)XT_TO_INT(radius), c);
}

void XT_DrawEllipse(uintptr_t cx, uintptr_t cy, uintptr_t rh, uintptr_t rv,
                    uintptr_t r, uintptr_t g, uintptr_t b, uintptr_t a) {
    Color c = {(unsigned char)XT_TO_INT(r), (unsigned char)XT_TO_INT(g),
               (unsigned char)XT_TO_INT(b), (unsigned char)XT_TO_INT(a)};
    DrawEllipse((int)XT_TO_INT(cx), (int)XT_TO_INT(cy),
                (float)XT_TO_INT(rh), (float)XT_TO_INT(rv), c);
}

void XT_DrawTriangle(uintptr_t x1, uintptr_t y1, uintptr_t x2, uintptr_t y2, uintptr_t x3, uintptr_t y3,
                     uintptr_t r, uintptr_t g, uintptr_t b, uintptr_t a) {
    Color c = {(unsigned char)XT_TO_INT(r), (unsigned char)XT_TO_INT(g),
               (unsigned char)XT_TO_INT(b), (unsigned char)XT_TO_INT(a)};
    Vector2 v1 = {(float)XT_TO_INT(x1), (float)XT_TO_INT(y1)};
    Vector2 v2 = {(float)XT_TO_INT(x2), (float)XT_TO_INT(y2)};
    Vector2 v3 = {(float)XT_TO_INT(x3), (float)XT_TO_INT(y3)};
    DrawTriangle(v1, v2, v3, c);
}

void XT_DrawPoly(uintptr_t cx, uintptr_t cy, uintptr_t sides, uintptr_t radius, uintptr_t rotation,
                 uintptr_t r, uintptr_t g, uintptr_t b, uintptr_t a) {
    Color c = {(unsigned char)XT_TO_INT(r), (unsigned char)XT_TO_INT(g),
               (unsigned char)XT_TO_INT(b), (unsigned char)XT_TO_INT(a)};
    Vector2 center = {(float)XT_TO_INT(cx), (float)XT_TO_INT(cy)};
    DrawPoly(center, (int)XT_TO_INT(sides), (float)XT_TO_INT(radius), (float)XT_TO_INT(rotation), c);
}

// ============================================================
// 文字绘制
// ============================================================

void XT_DrawText(uintptr_t text, uintptr_t x, uintptr_t y, uintptr_t fontSize,
                 uintptr_t r, uintptr_t g, uintptr_t b, uintptr_t a) {
    Color c = {(unsigned char)XT_TO_INT(r), (unsigned char)XT_TO_INT(g),
               (unsigned char)XT_TO_INT(b), (unsigned char)XT_TO_INT(a)};
    DrawText(xt_get_cstr(text), (int)XT_TO_INT(x), (int)XT_TO_INT(y),
             (int)XT_TO_INT(fontSize), c);
}

uintptr_t XT_MeasureText(uintptr_t text, uintptr_t fontSize) {
    return XT_FROM_INT(MeasureText(xt_get_cstr(text), (int)XT_TO_INT(fontSize)));
}

void XT_DrawFPS(uintptr_t x, uintptr_t y) {
    DrawFPS((int)XT_TO_INT(x), (int)XT_TO_INT(y));
}

// ============================================================
// 纹理与图像
// ============================================================

// 返回纹理指针（i64）
uintptr_t XT_LoadTexture(uintptr_t filename) {
    Texture2D tex = LoadTexture(xt_get_cstr(filename));
    Texture2D* p = (Texture2D*)calloc(1, sizeof(Texture2D));
    *p = tex;
    return (uintptr_t)p;
}

void XT_UnloadTexture(uintptr_t texPtr) {
    if (IS_PTR(texPtr) && texPtr != 0) {
        Texture2D* p = (Texture2D*)texPtr;
        UnloadTexture(*p);
        free((void*)p);
    }
}

void XT_DrawTexture(uintptr_t texPtr, uintptr_t x, uintptr_t y, uintptr_t tint_r, uintptr_t tint_g, uintptr_t tint_b, uintptr_t tint_a) {
    if (IS_PTR(texPtr) && texPtr != 0) {
        Texture2D* p = (Texture2D*)texPtr;
        Color tint = {(unsigned char)XT_TO_INT(tint_r), (unsigned char)XT_TO_INT(tint_g),
                      (unsigned char)XT_TO_INT(tint_b), (unsigned char)XT_TO_INT(tint_a)};
        DrawTexture(*p, (int)XT_TO_INT(x), (int)XT_TO_INT(y), tint);
    }
}

void XT_DrawTextureEx(uintptr_t texPtr, uintptr_t x, uintptr_t y, uintptr_t rotation, uintptr_t scale,
                      uintptr_t tint_r, uintptr_t tint_g, uintptr_t tint_b, uintptr_t tint_a) {
    if (IS_PTR(texPtr) && texPtr != 0) {
        Texture2D* p = (Texture2D*)texPtr;
        Color tint = {(unsigned char)XT_TO_INT(tint_r), (unsigned char)XT_TO_INT(tint_g),
                      (unsigned char)XT_TO_INT(tint_b), (unsigned char)XT_TO_INT(tint_a)};
        Rectangle src = {0, 0, (float)p->width, (float)p->height};
        Vector2 pos = {(float)XT_TO_INT(x), (float)XT_TO_INT(y)};
        Vector2 origin = {0, 0};
        DrawTexturePro(*p, src, (Rectangle){pos.x, pos.y, (float)p->width * XT_TO_INT(scale), (float)p->height * XT_TO_INT(scale)},
                       origin, (float)XT_TO_INT(rotation), tint);
    }
}

// ============================================================
// 输入
// ============================================================

uintptr_t XT_IsKeyDown(uintptr_t key) {
    return XT_FROM_BOOL(IsKeyDown((int)XT_TO_INT(key)));
}

uintptr_t XT_IsKeyPressed(uintptr_t key) {
    return XT_FROM_BOOL(IsKeyPressed((int)XT_TO_INT(key)));
}

uintptr_t XT_GetKeyPressed(void) {
    return XT_FROM_INT(GetKeyPressed());
}

uintptr_t XT_IsMouseButtonDown(uintptr_t button) {
    return XT_FROM_BOOL(IsMouseButtonDown((int)XT_TO_INT(button)));
}

uintptr_t XT_IsMouseButtonPressed(uintptr_t button) {
    return XT_FROM_BOOL(IsMouseButtonPressed((int)XT_TO_INT(button)));
}

uintptr_t XT_GetMouseX(void) {
    return XT_FROM_INT(GetMouseX());
}

uintptr_t XT_GetMouseY(void) {
    return XT_FROM_INT(GetMouseY());
}

uintptr_t XT_GetMouseWheelMove(void) {
    return XT_FROM_INT((int)GetMouseWheelMove());
}

// ============================================================
// 随机
// ============================================================

uintptr_t XT_GetRandomValue(uintptr_t min, uintptr_t max) {
    return XT_FROM_INT(GetRandomValue((int)XT_TO_INT(min), (int)XT_TO_INT(max)));
}

// ============================================================
// 计时
// ============================================================

uintptr_t XT_GetTime(void) {
    return xt_make_float(GetTime());
}

void XT_WaitTime(uintptr_t seconds) {
    WaitTime(xt_get_float(seconds));
}

// ============================================================
// 音频 (简化版)
// ============================================================

void XT_InitAudioDevice(void) {
    InitAudioDevice();
}

void XT_CloseAudioDevice(void) {
    CloseAudioDevice();
}

uintptr_t XT_LoadSound(uintptr_t filename) {
    Sound s = LoadSound(xt_get_cstr(filename));
    Sound* p = (Sound*)calloc(1, sizeof(Sound));
    *p = s;
    return (uintptr_t)p;
}

void XT_UnloadSound(uintptr_t sndPtr) {
    if (IS_PTR(sndPtr) && sndPtr != 0) {
        Sound* p = (Sound*)sndPtr;
        UnloadSound(*p);
        free((void*)p);
    }
}

void XT_PlaySound(uintptr_t sndPtr) {
    if (IS_PTR(sndPtr) && sndPtr != 0) {
        PlaySound(*(Sound*)sndPtr);
    }
}

uintptr_t XT_LoadMusicStream(uintptr_t filename) {
    Music m = LoadMusicStream(xt_get_cstr(filename));
    Music* p = (Music*)calloc(1, sizeof(Music));
    *p = m;
    return (uintptr_t)p;
}

void XT_PlayMusicStream(uintptr_t musicPtr) {
    if (IS_PTR(musicPtr) && musicPtr != 0) {
        PlayMusicStream(*(Music*)musicPtr);
    }
}

void XT_UpdateMusicStream(uintptr_t musicPtr) {
    if (IS_PTR(musicPtr) && musicPtr != 0) {
        UpdateMusicStream(*(Music*)musicPtr);
    }
}

void XT_StopMusicStream(uintptr_t musicPtr) {
    if (IS_PTR(musicPtr) && musicPtr != 0) {
        StopMusicStream(*(Music*)musicPtr);
    }
}

void XT_UnloadMusicStream(uintptr_t musicPtr) {
    if (IS_PTR(musicPtr) && musicPtr != 0) {
        UnloadMusicStream(*(Music*)musicPtr);
        free((void*)musicPtr);
    }
}
