public func detect_wine() -> Optional<RustString> {
    { let val = __swift_bridge__$detect_wine(); if val != nil { return RustString(ptr: val!) } else { return nil } }()
}
public func wine_version() -> Optional<RustString> {
    { let val = __swift_bridge__$wine_version(); if val != nil { return RustString(ptr: val!) } else { return nil } }()
}
public func get_base_dir() -> RustString {
    RustString(ptr: __swift_bridge__$get_base_dir())
}
public func list_all_prefixes() -> RustVec<RustPrefix> {
    RustVec(ptr: __swift_bridge__$list_all_prefixes())
}
public func download_file<GenericToRustStr: ToRustStr>(_ url: GenericToRustStr, _ dest: GenericToRustStr) -> Bool {
    return dest.toRustStr({ destAsRustStr in
        return url.toRustStr({ urlAsRustStr in
        __swift_bridge__$download_file(urlAsRustStr, destAsRustStr)
    })
    })
}
public func get_wine_dir() -> RustString {
    RustString(ptr: __swift_bridge__$get_wine_dir())
}
public func get_gptk_dir() -> RustString {
    RustString(ptr: __swift_bridge__$get_gptk_dir())
}
public func install_wine<GenericToRustStr: ToRustStr>(_ url: GenericToRustStr) -> Bool {
    return url.toRustStr({ urlAsRustStr in
        __swift_bridge__$install_wine(urlAsRustStr)
    })
}
public func is_wine_installed() -> Bool {
    __swift_bridge__$is_wine_installed()
}
public func last_install_error() -> Bool {
    __swift_bridge__$last_install_error()
}
public func get_available_verbs() -> RustVec<RustString> {
    RustVec(ptr: __swift_bridge__$get_available_verbs())
}
public func detect_all_wine_backends() -> RustVec<WineBackendInfo> {
    RustVec(ptr: __swift_bridge__$detect_all_wine_backends())
}
public func get_best_wine_backend() -> Optional<WineBackendInfo> {
    { let val = __swift_bridge__$get_best_wine_backend(); if val != nil { return WineBackendInfo(ptr: val!) } else { return nil } }()
}
public func has_wine_msvcrt_bug() -> Bool {
    __swift_bridge__$has_wine_msvcrt_bug()
}
public func kill_process(_ pid: UInt32) -> Bool {
    __swift_bridge__$kill_process(pid)
}
public enum SwiftWindowsVersion {
    case Win10
    case Win11
}
extension SwiftWindowsVersion {
    func intoFfiRepr() -> __swift_bridge__$SwiftWindowsVersion {
        switch self {
            case SwiftWindowsVersion.Win10:
                return __swift_bridge__$SwiftWindowsVersion(tag: __swift_bridge__$SwiftWindowsVersion$Win10)
            case SwiftWindowsVersion.Win11:
                return __swift_bridge__$SwiftWindowsVersion(tag: __swift_bridge__$SwiftWindowsVersion$Win11)
        }
    }
}
extension __swift_bridge__$SwiftWindowsVersion {
    func intoSwiftRepr() -> SwiftWindowsVersion {
        switch self.tag {
            case __swift_bridge__$SwiftWindowsVersion$Win10:
                return SwiftWindowsVersion.Win10
            case __swift_bridge__$SwiftWindowsVersion$Win11:
                return SwiftWindowsVersion.Win11
            default:
                fatalError("Unreachable")
        }
    }
}
extension __swift_bridge__$Option$SwiftWindowsVersion {
    @inline(__always)
    func intoSwiftRepr() -> Optional<SwiftWindowsVersion> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }
    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<SwiftWindowsVersion>) -> __swift_bridge__$Option$SwiftWindowsVersion {
        if let v = val {
            return __swift_bridge__$Option$SwiftWindowsVersion(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$SwiftWindowsVersion(is_some: false, val: __swift_bridge__$SwiftWindowsVersion())
        }
    }
}
extension SwiftWindowsVersion: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_SwiftWindowsVersion$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_SwiftWindowsVersion$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: Self) {
        __swift_bridge__$Vec_SwiftWindowsVersion$push(vecPtr, value.intoFfiRepr())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let maybeEnum = __swift_bridge__$Vec_SwiftWindowsVersion$pop(vecPtr)
        return maybeEnum.intoSwiftRepr()
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<Self> {
        let maybeEnum = __swift_bridge__$Vec_SwiftWindowsVersion$get(vecPtr, index)
        return maybeEnum.intoSwiftRepr()
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<Self> {
        let maybeEnum = __swift_bridge__$Vec_SwiftWindowsVersion$get_mut(vecPtr, index)
        return maybeEnum.intoSwiftRepr()
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<Self> {
        UnsafePointer<Self>(OpaquePointer(__swift_bridge__$Vec_SwiftWindowsVersion$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_SwiftWindowsVersion$len(vecPtr)
    }
}
public enum SwiftGraphicsBackend {
    case D3DMetal
    case DXVK
}
extension SwiftGraphicsBackend {
    func intoFfiRepr() -> __swift_bridge__$SwiftGraphicsBackend {
        switch self {
            case SwiftGraphicsBackend.D3DMetal:
                return __swift_bridge__$SwiftGraphicsBackend(tag: __swift_bridge__$SwiftGraphicsBackend$D3DMetal)
            case SwiftGraphicsBackend.DXVK:
                return __swift_bridge__$SwiftGraphicsBackend(tag: __swift_bridge__$SwiftGraphicsBackend$DXVK)
        }
    }
}
extension __swift_bridge__$SwiftGraphicsBackend {
    func intoSwiftRepr() -> SwiftGraphicsBackend {
        switch self.tag {
            case __swift_bridge__$SwiftGraphicsBackend$D3DMetal:
                return SwiftGraphicsBackend.D3DMetal
            case __swift_bridge__$SwiftGraphicsBackend$DXVK:
                return SwiftGraphicsBackend.DXVK
            default:
                fatalError("Unreachable")
        }
    }
}
extension __swift_bridge__$Option$SwiftGraphicsBackend {
    @inline(__always)
    func intoSwiftRepr() -> Optional<SwiftGraphicsBackend> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }
    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<SwiftGraphicsBackend>) -> __swift_bridge__$Option$SwiftGraphicsBackend {
        if let v = val {
            return __swift_bridge__$Option$SwiftGraphicsBackend(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$SwiftGraphicsBackend(is_some: false, val: __swift_bridge__$SwiftGraphicsBackend())
        }
    }
}
extension SwiftGraphicsBackend: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_SwiftGraphicsBackend$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_SwiftGraphicsBackend$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: Self) {
        __swift_bridge__$Vec_SwiftGraphicsBackend$push(vecPtr, value.intoFfiRepr())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let maybeEnum = __swift_bridge__$Vec_SwiftGraphicsBackend$pop(vecPtr)
        return maybeEnum.intoSwiftRepr()
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<Self> {
        let maybeEnum = __swift_bridge__$Vec_SwiftGraphicsBackend$get(vecPtr, index)
        return maybeEnum.intoSwiftRepr()
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<Self> {
        let maybeEnum = __swift_bridge__$Vec_SwiftGraphicsBackend$get_mut(vecPtr, index)
        return maybeEnum.intoSwiftRepr()
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<Self> {
        UnsafePointer<Self>(OpaquePointer(__swift_bridge__$Vec_SwiftGraphicsBackend$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_SwiftGraphicsBackend$len(vecPtr)
    }
}
public enum SwiftSyncMode {
    case Default
    case ESync
    case MSync
}
extension SwiftSyncMode {
    func intoFfiRepr() -> __swift_bridge__$SwiftSyncMode {
        switch self {
            case SwiftSyncMode.Default:
                return __swift_bridge__$SwiftSyncMode(tag: __swift_bridge__$SwiftSyncMode$Default)
            case SwiftSyncMode.ESync:
                return __swift_bridge__$SwiftSyncMode(tag: __swift_bridge__$SwiftSyncMode$ESync)
            case SwiftSyncMode.MSync:
                return __swift_bridge__$SwiftSyncMode(tag: __swift_bridge__$SwiftSyncMode$MSync)
        }
    }
}
extension __swift_bridge__$SwiftSyncMode {
    func intoSwiftRepr() -> SwiftSyncMode {
        switch self.tag {
            case __swift_bridge__$SwiftSyncMode$Default:
                return SwiftSyncMode.Default
            case __swift_bridge__$SwiftSyncMode$ESync:
                return SwiftSyncMode.ESync
            case __swift_bridge__$SwiftSyncMode$MSync:
                return SwiftSyncMode.MSync
            default:
                fatalError("Unreachable")
        }
    }
}
extension __swift_bridge__$Option$SwiftSyncMode {
    @inline(__always)
    func intoSwiftRepr() -> Optional<SwiftSyncMode> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }
    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<SwiftSyncMode>) -> __swift_bridge__$Option$SwiftSyncMode {
        if let v = val {
            return __swift_bridge__$Option$SwiftSyncMode(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$SwiftSyncMode(is_some: false, val: __swift_bridge__$SwiftSyncMode())
        }
    }
}
extension SwiftSyncMode: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_SwiftSyncMode$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_SwiftSyncMode$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: Self) {
        __swift_bridge__$Vec_SwiftSyncMode$push(vecPtr, value.intoFfiRepr())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let maybeEnum = __swift_bridge__$Vec_SwiftSyncMode$pop(vecPtr)
        return maybeEnum.intoSwiftRepr()
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<Self> {
        let maybeEnum = __swift_bridge__$Vec_SwiftSyncMode$get(vecPtr, index)
        return maybeEnum.intoSwiftRepr()
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<Self> {
        let maybeEnum = __swift_bridge__$Vec_SwiftSyncMode$get_mut(vecPtr, index)
        return maybeEnum.intoSwiftRepr()
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<Self> {
        UnsafePointer<Self>(OpaquePointer(__swift_bridge__$Vec_SwiftSyncMode$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_SwiftSyncMode$len(vecPtr)
    }
}
public enum SwiftWineBackend {
    case Auto
    case WineHQ
    case GPTK
    case CrossOver
    case Custom
}
extension SwiftWineBackend {
    func intoFfiRepr() -> __swift_bridge__$SwiftWineBackend {
        switch self {
            case SwiftWineBackend.Auto:
                return __swift_bridge__$SwiftWineBackend(tag: __swift_bridge__$SwiftWineBackend$Auto)
            case SwiftWineBackend.WineHQ:
                return __swift_bridge__$SwiftWineBackend(tag: __swift_bridge__$SwiftWineBackend$WineHQ)
            case SwiftWineBackend.GPTK:
                return __swift_bridge__$SwiftWineBackend(tag: __swift_bridge__$SwiftWineBackend$GPTK)
            case SwiftWineBackend.CrossOver:
                return __swift_bridge__$SwiftWineBackend(tag: __swift_bridge__$SwiftWineBackend$CrossOver)
            case SwiftWineBackend.Custom:
                return __swift_bridge__$SwiftWineBackend(tag: __swift_bridge__$SwiftWineBackend$Custom)
        }
    }
}
extension __swift_bridge__$SwiftWineBackend {
    func intoSwiftRepr() -> SwiftWineBackend {
        switch self.tag {
            case __swift_bridge__$SwiftWineBackend$Auto:
                return SwiftWineBackend.Auto
            case __swift_bridge__$SwiftWineBackend$WineHQ:
                return SwiftWineBackend.WineHQ
            case __swift_bridge__$SwiftWineBackend$GPTK:
                return SwiftWineBackend.GPTK
            case __swift_bridge__$SwiftWineBackend$CrossOver:
                return SwiftWineBackend.CrossOver
            case __swift_bridge__$SwiftWineBackend$Custom:
                return SwiftWineBackend.Custom
            default:
                fatalError("Unreachable")
        }
    }
}
extension __swift_bridge__$Option$SwiftWineBackend {
    @inline(__always)
    func intoSwiftRepr() -> Optional<SwiftWineBackend> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }
    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<SwiftWineBackend>) -> __swift_bridge__$Option$SwiftWineBackend {
        if let v = val {
            return __swift_bridge__$Option$SwiftWineBackend(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$SwiftWineBackend(is_some: false, val: __swift_bridge__$SwiftWineBackend())
        }
    }
}
extension SwiftWineBackend: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_SwiftWineBackend$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_SwiftWineBackend$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: Self) {
        __swift_bridge__$Vec_SwiftWineBackend$push(vecPtr, value.intoFfiRepr())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let maybeEnum = __swift_bridge__$Vec_SwiftWineBackend$pop(vecPtr)
        return maybeEnum.intoSwiftRepr()
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<Self> {
        let maybeEnum = __swift_bridge__$Vec_SwiftWineBackend$get(vecPtr, index)
        return maybeEnum.intoSwiftRepr()
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<Self> {
        let maybeEnum = __swift_bridge__$Vec_SwiftWineBackend$get_mut(vecPtr, index)
        return maybeEnum.intoSwiftRepr()
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<Self> {
        UnsafePointer<Self>(OpaquePointer(__swift_bridge__$Vec_SwiftWineBackend$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_SwiftWineBackend$len(vecPtr)
    }
}

public class RustPrefix: RustPrefixRefMut {
    var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$RustPrefix$_free(ptr)
        }
    }
}
extension RustPrefix {
    public convenience init?<GenericToRustStr: ToRustStr>(_ name: GenericToRustStr) {
        guard let val = name.toRustStr({ nameAsRustStr in
            __swift_bridge__$RustPrefix$new_prefix(nameAsRustStr)
        }) else { return nil }; self.init(ptr: val)
    }
}
public class RustPrefixRefMut: RustPrefixRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
extension RustPrefixRefMut {
    public func set_windows_version(_ version: SwiftWindowsVersion) {
        __swift_bridge__$RustPrefix$set_windows_version(ptr, version.intoFfiRepr())
    }

    public func set_graphics_backend(_ backend: SwiftGraphicsBackend) {
        __swift_bridge__$RustPrefix$set_graphics_backend(ptr, backend.intoFfiRepr())
    }

    public func set_sync_mode(_ mode: SwiftSyncMode) {
        __swift_bridge__$RustPrefix$set_sync_mode(ptr, mode.intoFfiRepr())
    }

    public func set_metal_fx(_ enabled: Bool) {
        __swift_bridge__$RustPrefix$set_metal_fx(ptr, enabled)
    }

    public func set_dxvk_hud(_ enabled: Bool) {
        __swift_bridge__$RustPrefix$set_dxvk_hud(ptr, enabled)
    }

    public func set_wine_path<GenericToRustStr: ToRustStr>(_ path: GenericToRustStr) {
        path.toRustStr({ pathAsRustStr in
            __swift_bridge__$RustPrefix$set_wine_path(ptr, pathAsRustStr)
        })
    }

    public func set_wine_backend(_ backend: SwiftWineBackend) {
        __swift_bridge__$RustPrefix$set_wine_backend(ptr, backend.intoFfiRepr())
    }

    public func set_hid_controllers(_ enabled: Bool) {
        __swift_bridge__$RustPrefix$set_hid_controllers(ptr, enabled)
    }

    public func set_reduce_wine_debug(_ enabled: Bool) {
        __swift_bridge__$RustPrefix$set_reduce_wine_debug(ptr, enabled)
    }
}
public class RustPrefixRef {
    var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension RustPrefixRef {
    public func get_id() -> RustString {
        RustString(ptr: __swift_bridge__$RustPrefix$get_id(ptr))
    }

    public func get_name() -> RustString {
        RustString(ptr: __swift_bridge__$RustPrefix$get_name(ptr))
    }

    public func get_path() -> RustString {
        RustString(ptr: __swift_bridge__$RustPrefix$get_path(ptr))
    }

    public func get_windows_version() -> SwiftWindowsVersion {
        __swift_bridge__$RustPrefix$get_windows_version(ptr).intoSwiftRepr()
    }

    public func get_graphics_backend() -> SwiftGraphicsBackend {
        __swift_bridge__$RustPrefix$get_graphics_backend(ptr).intoSwiftRepr()
    }

    public func get_sync_mode() -> SwiftSyncMode {
        __swift_bridge__$RustPrefix$get_sync_mode(ptr).intoSwiftRepr()
    }

    public func get_metal_fx() -> Bool {
        __swift_bridge__$RustPrefix$get_metal_fx(ptr)
    }

    public func get_dxvk_hud() -> Bool {
        __swift_bridge__$RustPrefix$get_dxvk_hud(ptr)
    }

    public func get_wine_path() -> Optional<RustString> {
        { let val = __swift_bridge__$RustPrefix$get_wine_path(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func get_wine_backend() -> SwiftWineBackend {
        __swift_bridge__$RustPrefix$get_wine_backend(ptr).intoSwiftRepr()
    }

    public func get_hid_controllers() -> Bool {
        __swift_bridge__$RustPrefix$get_hid_controllers(ptr)
    }

    public func get_reduce_wine_debug() -> Bool {
        __swift_bridge__$RustPrefix$get_reduce_wine_debug(ptr)
    }

    public func save() -> Bool {
        __swift_bridge__$RustPrefix$save(ptr)
    }

    public func delete_prefix() -> Bool {
        __swift_bridge__$RustPrefix$delete_prefix(ptr)
    }

    public func reinit_prefix() -> RustString {
        RustString(ptr: __swift_bridge__$RustPrefix$reinit_prefix(ptr))
    }

    public func list_executables() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$RustPrefix$list_executables(ptr))
    }

    public func run_program<GenericToRustStr: ToRustStr>(_ program_path: GenericToRustStr) -> Bool {
        return program_path.toRustStr({ program_pathAsRustStr in
            __swift_bridge__$RustPrefix$run_program(ptr, program_pathAsRustStr)
        })
    }

    public func launch_program<GenericToRustStr: ToRustStr>(_ program_path: GenericToRustStr) -> UInt32 {
        return program_path.toRustStr({ program_pathAsRustStr in
            __swift_bridge__$RustPrefix$launch_program(ptr, program_pathAsRustStr)
        })
    }

    public func init_prefix() -> Bool {
        __swift_bridge__$RustPrefix$init_prefix(ptr)
    }

    public func install_steam() -> RustString {
        RustString(ptr: __swift_bridge__$RustPrefix$install_steam(ptr))
    }

    public func launch_steam() -> UInt32 {
        __swift_bridge__$RustPrefix$launch_steam(ptr)
    }

    public func run_winetricks<GenericToRustStr: ToRustStr>(_ verb: GenericToRustStr) -> RustString {
        return verb.toRustStr({ verbAsRustStr in
            RustString(ptr: __swift_bridge__$RustPrefix$run_winetricks(ptr, verbAsRustStr))
        })
    }

    public func find_steam_exe() -> Optional<RustString> {
        { let val = __swift_bridge__$RustPrefix$find_steam_exe(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }
}
extension RustPrefix: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_RustPrefix$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_RustPrefix$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: RustPrefix) {
        __swift_bridge__$Vec_RustPrefix$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_RustPrefix$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (RustPrefix(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<RustPrefixRef> {
        let pointer = __swift_bridge__$Vec_RustPrefix$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return RustPrefixRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<RustPrefixRefMut> {
        let pointer = __swift_bridge__$Vec_RustPrefix$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return RustPrefixRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<RustPrefixRef> {
        UnsafePointer<RustPrefixRef>(OpaquePointer(__swift_bridge__$Vec_RustPrefix$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_RustPrefix$len(vecPtr)
    }
}


public class WineBackendInfo: WineBackendInfoRefMut {
    var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$WineBackendInfo$_free(ptr)
        }
    }
}
public class WineBackendInfoRefMut: WineBackendInfoRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class WineBackendInfoRef {
    var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension WineBackendInfoRef {
    public func get_backend_name() -> RustString {
        RustString(ptr: __swift_bridge__$WineBackendInfo$get_backend_name(ptr))
    }

    public func get_wine_path() -> RustString {
        RustString(ptr: __swift_bridge__$WineBackendInfo$get_wine_path(ptr))
    }

    public func get_version() -> Optional<RustString> {
        { let val = __swift_bridge__$WineBackendInfo$get_version(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func has_wo64_support() -> Bool {
        __swift_bridge__$WineBackendInfo$has_wo64_support(ptr)
    }
}
extension WineBackendInfo: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_WineBackendInfo$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_WineBackendInfo$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: WineBackendInfo) {
        __swift_bridge__$Vec_WineBackendInfo$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_WineBackendInfo$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (WineBackendInfo(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<WineBackendInfoRef> {
        let pointer = __swift_bridge__$Vec_WineBackendInfo$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return WineBackendInfoRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<WineBackendInfoRefMut> {
        let pointer = __swift_bridge__$Vec_WineBackendInfo$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return WineBackendInfoRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<WineBackendInfoRef> {
        UnsafePointer<WineBackendInfoRef>(OpaquePointer(__swift_bridge__$Vec_WineBackendInfo$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_WineBackendInfo$len(vecPtr)
    }
}



