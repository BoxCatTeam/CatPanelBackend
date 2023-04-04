const {core} = Deno[Deno.internal];
const {ops} = core;

globalThis.cp = {};

globalThis.info = (info_obj) => {
    globalThis.cp.info = () => transform_fn_field_obj(info_obj);
    ops.op_reg_info(cp.info());
};

globalThis.installer = (install_func) => {
    globalThis.cp.installer = install_func;
};

function transform_fn_field_obj(obj) {
    for (const k in obj) {
        if (typeof obj[k] == "function") obj[k] = obj[k]()
    }
    return obj;
}