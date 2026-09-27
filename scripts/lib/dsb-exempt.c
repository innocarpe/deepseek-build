// dsb-exempt — run a command with this app as its macOS responsible process.
//
//   dsb-exempt <command> [args...]
//   dsb-exempt --check
//
// syspolicyd assesses every freshly built executable and dylib on its first
// exec/load (measured on this machine: 300-590 ms for a small binary, 10 s for
// a 512 MB debug build). Apps listed under System Settings > Privacy &
// Security > Developer Tools are exempt, but the exemption follows the
// *responsible* process. Orca spawns its terminal daemon with responsibility
// disclaimed, so every process in an Orca tab is responsible for itself and
// the exemption never applies there, not even after a restart.
//
// This binary re-spawns itself with responsibility disclaimed, so the inner
// copy is its own responsible process (identity: this app bundle, the one
// registered under Developer Tools). The inner copy spawns the command as a
// normal child, which inherits it as responsible, and waits. It must stay
// alive: after an exec the identity would be the command's, not this app's.
// stdin/stdout/stderr pass straight through, so a TUI runs in the same tab.
#include <errno.h>
#include <limits.h>
#include <mach-o/dyld.h>
#include <signal.h>
#include <spawn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

extern char **environ;
int responsibility_spawnattrs_setdisclaim(posix_spawnattr_t *attrs, int disclaim);
int responsibility_get_pid_responsible_for_pid(pid_t pid);

#define INNER_ENV "DSB_EXEMPT_INNER"

static int wait_child(pid_t pid) {
    int status;
    while (waitpid(pid, &status, 0) < 0) {
        if (errno != EINTR) {
            perror("dsb-exempt: waitpid");
            return 127;
        }
    }
    if (WIFEXITED(status)) return WEXITSTATUS(status);
    if (WIFSIGNALED(status)) {
        int sig = WTERMSIG(status);
        signal(sig, SIG_DFL);
        kill(getpid(), sig);
        return 128 + sig;
    }
    return 127;
}

// The terminal delivers ^C / ^\ to the whole foreground process group. The
// command decides what they mean; the runners only wait for it.
static void ignore_terminal_signals(void) {
    signal(SIGINT, SIG_IGN);
    signal(SIGQUIT, SIG_IGN);
}

static int outer(char **argv) {
    char self[PATH_MAX];
    uint32_t size = sizeof self;
    if (_NSGetExecutablePath(self, &size) != 0) {
        fprintf(stderr, "dsb-exempt: executable path too long\n");
        return 127;
    }
    posix_spawnattr_t attr;
    posix_spawnattr_init(&attr);
    if (responsibility_spawnattrs_setdisclaim(&attr, 1) != 0) {
        fprintf(stderr, "dsb-exempt: could not disclaim responsibility\n");
        return 127;
    }
    setenv(INNER_ENV, "1", 1);
    pid_t pid;
    int rc = posix_spawn(&pid, self, NULL, &attr, argv, environ);
    posix_spawnattr_destroy(&attr);
    if (rc != 0) {
        fprintf(stderr, "dsb-exempt: spawn %s: %s\n", self, strerror(rc));
        return 127;
    }
    ignore_terminal_signals();
    return wait_child(pid);
}

static int check(void) {
    pid_t pid;
    char *const child_argv[] = {"/bin/sleep", "0.3", NULL};
    int rc = posix_spawn(&pid, "/bin/sleep", NULL, NULL, child_argv, environ);
    if (rc != 0) {
        fprintf(stderr, "dsb-exempt: spawn /bin/sleep: %s\n", strerror(rc));
        return 127;
    }
    pid_t me = getpid();
    int mine = responsibility_get_pid_responsible_for_pid(me);
    int theirs = responsibility_get_pid_responsible_for_pid(pid);
    wait_child(pid);
    int ok = mine == me && theirs == me;
    printf("runner pid %d responsible %d; child pid %d responsible %d -> %s\n", me, mine, pid, theirs,
           ok ? "ok: children are attributed to this app" : "NOT attributed to this app");
    return ok ? 0 : 1;
}

static int inner(char **argv) {
    unsetenv(INNER_ENV);
    if (strcmp(argv[1], "--check") == 0) return check();
    pid_t pid;
    int rc = posix_spawnp(&pid, argv[1], NULL, NULL, argv + 1, environ);
    if (rc != 0) {
        fprintf(stderr, "dsb-exempt: %s: %s\n", argv[1], strerror(rc));
        return 127;
    }
    ignore_terminal_signals();
    return wait_child(pid);
}

int main(int argc, char **argv) {
    if (argc < 2 || strcmp(argv[1], "-h") == 0 || strcmp(argv[1], "--help") == 0) {
        fprintf(stderr, "usage: dsb-exempt <command> [args...]\n       dsb-exempt --check\n");
        return argc < 2 ? 2 : 0;
    }
    const char *flag = getenv(INNER_ENV);
    if (flag != NULL && strcmp(flag, "1") == 0) return inner(argv);
    return outer(argv);
}
