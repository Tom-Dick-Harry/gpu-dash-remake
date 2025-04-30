/*
 * Windows System Information Monitor
 * Based on concepts from the psutil python lib
 */

 #include <stdio.h>
 #include <windows.h>
 #include <psapi.h>
 #include <powrprof.h>
 
 #include <winternl.h>
 
 // Expose telemetry struct and function for FFI
 #include <stdint.h>
 
 
 // Conversion macros (from psutil)
 #define HI_T (float)1.0e-7
 #define LO_T (float)429.4967296
 
 typedef NTSTATUS (NTAPI *_NtQuerySystemInformation)(
    SYSTEM_INFORMATION_CLASS SystemInformationClass,
    PVOID SystemInformation,
    ULONG SystemInformationLength,
    PULONG ReturnLength
);
 
 // Global variables
 _NtQuerySystemInformation pNtQuerySystemInformation;
 
 // Initialize the required function pointers
 int init_function_pointers() {
    HMODULE ntdll = GetModuleHandle("ntdll.dll");
    if (ntdll == NULL) {
        printf("Failed to get handle to ntdll.dll\n");
        return 0;
    }

    pNtQuerySystemInformation = (_NtQuerySystemInformation)GetProcAddress(
        ntdll, "NtQuerySystemInformation");
    if (pNtQuerySystemInformation == NULL) {
        printf("Failed to get address of NtQuerySystemInformation\n");
        return 0;
    }

    return 1;
}
 
 // Get system uptime
 double get_uptime() {
     double uptime_seconds;
     ULONGLONG interrupt_time100ns = 0;
     
     // On Windows 7+ use QueryInterruptTime for more accurate results
     BOOL (WINAPI *QueryInterruptTimePtr)(PULONGLONG) = NULL;
     HMODULE kernel32 = GetModuleHandle("kernel32.dll");
     
     if (kernel32 != NULL) {
         QueryInterruptTimePtr = (void*)GetProcAddress(kernel32, "QueryInterruptTime");
     }
     
     if (QueryInterruptTimePtr != NULL) {
         QueryInterruptTimePtr(&interrupt_time100ns);
         // Convert from 100-nanosecond to seconds
         uptime_seconds = interrupt_time100ns / 10000000.0;
     } else {
         // Fallback to GetTickCount64 (available since Windows Vista)
         uptime_seconds = (double)GetTickCount64() / 1000.0;
     }
     
     return uptime_seconds;
 }
 
 // Get CPU times (user, system, idle)
 int get_cpu_times(double *user, double *system, double *idle) {
    FILETIME idle_time, kernel_time, user_time;

    if (!GetSystemTimes(&idle_time, &kernel_time, &user_time)) {
        printf("Failed to get system times: %lu\n", GetLastError());
        return 0;
    }

    double idle_val = (double)((HI_T * idle_time.dwHighDateTime) +
                               (LO_T * idle_time.dwLowDateTime));
    double user_val = (double)((HI_T * user_time.dwHighDateTime) +
                               (LO_T * user_time.dwLowDateTime));
    double kernel_val = (double)((HI_T * kernel_time.dwHighDateTime) +
                                 (LO_T * kernel_time.dwLowDateTime));

    *idle = idle_val;
    *user = user_val;
    // Kernel time includes idle time. Return only busy kernel time.
    *system = kernel_val - idle_val;
    return 1;
}
 
 // Get memory information
 int get_memory_info(MEMORYSTATUSEX *mem_info) {
     mem_info->dwLength = sizeof(MEMORYSTATUSEX);
     if (!GlobalMemoryStatusEx(mem_info)) {
         printf("Failed to get memory status: %lu\n", GetLastError());
         return 0;
     }
     return 1;
 }
 
 // Get disk usage
 int get_disk_usage(const char *path, ULARGE_INTEGER *free_bytes, 
                   ULARGE_INTEGER *total_bytes, ULARGE_INTEGER *total_free_bytes) {
     if (!GetDiskFreeSpaceExA(
             path,
             free_bytes,
             total_bytes,
             total_free_bytes)) {
         printf("Failed to get disk space for %s: %lu\n", path, GetLastError());
         return 0;
     }
     return 1;
 }
 
 // Get per-CPU statistics
 int get_per_cpu_times() {
    NTSTATUS status;
    PSYSTEM_PROCESSOR_PERFORMANCE_INFORMATION sppi = NULL;
    ULONG return_length;
    int ncpus = 0;
    int i;

    SYSTEM_INFO sysinfo;
    GetSystemInfo(&sysinfo);
    ncpus = sysinfo.dwNumberOfProcessors;

    sppi = (PSYSTEM_PROCESSOR_PERFORMANCE_INFORMATION)malloc(
        ncpus * sizeof(SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION));
    if (sppi == NULL) {
        printf("Failed to allocate memory\n");
        return 0;
    }

    status = pNtQuerySystemInformation(
        SystemProcessorPerformanceInformation,
        sppi,
        ncpus * sizeof(SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION),
        &return_length);

    if (status != 0) {
        printf("NtQuerySystemInformation failed with status %lx\n", status);
        free(sppi);
        return 0;
    }

    printf("CPU Times per processor:\n");
    for (i = 0; i < ncpus; i++) {
        double user, idle, kernel, system;

        user = (double)((HI_T * sppi[i].UserTime.HighPart) +
                        (LO_T * sppi[i].UserTime.LowPart));
        idle = (double)((HI_T * sppi[i].IdleTime.HighPart) +
                        (LO_T * sppi[i].IdleTime.LowPart));
        kernel = (double)((HI_T * sppi[i].KernelTime.HighPart) +
                          (LO_T * sppi[i].KernelTime.LowPart));
        // Kernel time includes idle time on Windows
        system = kernel - idle;

        printf("  CPU %d: User=%.2fs System=%.2fs Idle=%.2fs\n",
               i, user, system, idle);
    }

    free(sppi);
    return 1;
}

#ifdef __cplusplus
 extern "C" {
 #endif

 #define MAX_DISKS 26

 typedef struct {
     char mount[4]; // e.g. "C:\"
     unsigned long long total_space;
     unsigned long long free_space;
     unsigned long long used_space;
 } DiskInfo;

 typedef struct {
     unsigned long long total_phys;
     unsigned long long free_phys;
     unsigned long memory_load;
     unsigned long long total_virtual;
     unsigned long long free_virtual;
 } MemoryInfo;

 typedef struct {
     double uptime;
     double user_time;
     double system_time;
     double idle_time;
     MemoryInfo mem;
     DiskInfo disks[MAX_DISKS];
     int num_disks;
 } TelemetryInfoFull;

 __declspec(dllexport) int sysinfo_get_telemetry_full(TelemetryInfoFull* out) {
     if (!out) return 0;
     out->uptime = get_uptime();
     if (!get_cpu_times(&out->user_time, &out->system_time, &out->idle_time)) {
         return 0;
     }
     // Memory
     MEMORYSTATUSEX mem_info;
     if (!get_memory_info(&mem_info)) {
         return 0;
     }
     out->mem.total_phys = mem_info.ullTotalPhys;
     out->mem.free_phys = mem_info.ullAvailPhys;
     out->mem.memory_load = mem_info.dwMemoryLoad;
     out->mem.total_virtual = mem_info.ullTotalVirtual;
     out->mem.free_virtual = mem_info.ullAvailVirtual;
     // Disks
     out->num_disks = 0;
     char drive[4] = "A:\\";
     DWORD drives = GetLogicalDrives();
     for (int i = 0; i < 26; i++) {
         if (drives & (1 << i)) {
             drive[0] = 'A' + i;
             ULARGE_INTEGER free_bytes, total_bytes, total_free_bytes;
             if (get_disk_usage(drive, &free_bytes, &total_bytes, &total_free_bytes)) {
                 DiskInfo* d = &out->disks[out->num_disks++];
                 snprintf(d->mount, sizeof(d->mount), "%s", drive);
                 d->total_space = total_bytes.QuadPart;
                 d->free_space = free_bytes.QuadPart;
                 d->used_space = total_bytes.QuadPart - free_bytes.QuadPart;
             }
         }
     }
     return 1;
 }

 #ifdef __cplusplus
 }
 #endif
 
 // Main function to display system information
 int main() {
     // Initialize function pointers
     if (!init_function_pointers()) {
         printf("Failed to initialize function pointers. Exiting.\n");
         return 1;
     }
 
     // Display system uptime
     double uptime = get_uptime();
     printf("System Uptime: %.2f seconds (%.2f hours)\n", uptime, uptime / 3600.0);
 
     // Display CPU information
     double user_time, system_time, idle_time;
     if (get_cpu_times(&user_time, &system_time, &idle_time)) {
         printf("CPU Times (total):\n");
         printf("  User   : %.2f seconds\n", user_time);
         printf("  System : %.2f seconds\n", system_time);
         printf("  Idle   : %.2f seconds\n", idle_time);
     }
 
     // Display per-CPU information
     get_per_cpu_times();
 
     // Display memory information
     MEMORYSTATUSEX mem_info;
     if (get_memory_info(&mem_info)) {
         printf("Memory Information:\n");
         printf("  Total Physical: %lld MB\n", mem_info.ullTotalPhys / (1024 * 1024));
         printf("  Free Physical : %lld MB\n", mem_info.ullAvailPhys / (1024 * 1024));
         printf("  Memory Load   : %ld%%\n", mem_info.dwMemoryLoad);
         printf("  Total Virtual : %lld MB\n", mem_info.ullTotalVirtual / (1024 * 1024));
         printf("  Free Virtual  : %lld MB\n", mem_info.ullAvailVirtual / (1024 * 1024));
     }
 
     // Display disk usage for C: drive
     ULARGE_INTEGER free_bytes, total_bytes, total_free_bytes;
     if (get_disk_usage("C:\\", &free_bytes, &total_bytes, &total_free_bytes)) {
         printf("Disk Usage (C:):\n");
         printf("  Total Space: %llu GB\n", total_bytes.QuadPart / (1024 * 1024 * 1024));
         printf("  Free Space : %llu GB\n", free_bytes.QuadPart / (1024 * 1024 * 1024));
         printf("  Used Space : %llu GB\n", 
                (total_bytes.QuadPart - free_bytes.QuadPart) / (1024 * 1024 * 1024));
     }
 
     return 0;
 }
