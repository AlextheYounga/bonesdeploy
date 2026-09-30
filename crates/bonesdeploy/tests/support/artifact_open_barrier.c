#define _GNU_SOURCE

#include <dlfcn.h>
#include <fcntl.h>
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>
#include <sys/syscall.h>
#include <unistd.h>

typedef int (*OpenFunction)(const char *, int, mode_t);
typedef int (*OpenAtFunction)(int, const char *, int, mode_t);

static int matches_barrier_target(const char *path) {
    const char *target_file = getenv("ARTIFACT_BARRIER_TARGET_FILE");
    if (target_file == NULL || path == NULL) {
        return 0;
    }

    char target[4096];
    int fd = syscall(SYS_openat, AT_FDCWD, target_file, O_RDONLY);
    if (fd < 0) {
        return 0;
    }
    ssize_t length = syscall(SYS_read, fd, target, sizeof(target) - 1);
    syscall(SYS_close, fd);
    if (length <= 0) {
        return 0;
    }
    target[length] = '\0';
    target[strcspn(target, "\n")] = '\0';
    return strcmp(path, target) == 0;
}

static void wait_for_release(const char *path) {
    if (!matches_barrier_target(path)) {
        return;
    }

    const char *ready_file = getenv("ARTIFACT_BARRIER_READY_FILE");
    const char *release_file = getenv("ARTIFACT_BARRIER_RELEASE_FILE");
    if (ready_file == NULL || release_file == NULL) {
        return;
    }
    int ready = syscall(SYS_openat, AT_FDCWD, ready_file, O_WRONLY | O_CREAT | O_TRUNC, 0600);
    if (ready >= 0) {
        syscall(SYS_close, ready);
    }
    while (access(release_file, F_OK) != 0) {
        usleep(1000);
    }
}

int open(const char *path, int flags, ...) {
    static OpenFunction real_open;
    if (real_open == NULL) {
        real_open = (OpenFunction)dlsym(RTLD_NEXT, "open");
    }
    mode_t mode = 0;
    if ((flags & O_CREAT) != 0) {
        va_list arguments;
        va_start(arguments, flags);
        mode = va_arg(arguments, mode_t);
        va_end(arguments);
    }
    wait_for_release(path);
    return real_open(path, flags, mode);
}

int open64(const char *path, int flags, ...) {
    static OpenFunction real_open64;
    if (real_open64 == NULL) {
        real_open64 = (OpenFunction)dlsym(RTLD_NEXT, "open64");
    }
    mode_t mode = 0;
    if ((flags & O_CREAT) != 0) {
        va_list arguments;
        va_start(arguments, flags);
        mode = va_arg(arguments, mode_t);
        va_end(arguments);
    }
    wait_for_release(path);
    return real_open64(path, flags, mode);
}

int openat(int directory, const char *path, int flags, ...) {
    static OpenAtFunction real_openat;
    if (real_openat == NULL) {
        real_openat = (OpenAtFunction)dlsym(RTLD_NEXT, "openat");
    }
    mode_t mode = 0;
    if ((flags & O_CREAT) != 0) {
        va_list arguments;
        va_start(arguments, flags);
        mode = va_arg(arguments, mode_t);
        va_end(arguments);
    }
    wait_for_release(path);
    return real_openat(directory, path, flags, mode);
}
