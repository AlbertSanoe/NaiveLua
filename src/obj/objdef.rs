use crate::{
    info::lua::{ErrCode, NONE_OBJECT},
    obj::statedef::LuaState,
};

type Dt = u32;
pub type INT = i32; // integer
pub type FLT = f32; // float
pub type FFUNC = fn(&mut LuaState) -> usize;

pub const BASIC_TYPE_BIT: usize = 4;

pub const T_NUMBER: Dt = 1;
pub const T_LIGHT_USER_DATA: Dt = 2;
pub const T_BOOLEAN: Dt = 3;
pub const T_STRING: Dt = 4;
pub const T_NIL: Dt = 5;
pub const T_TABLE: Dt = 6;
pub const T_FUNCTION: Dt = 7;
pub const T_THREAD: Dt = 8;
pub const T_NONE: Dt = 9;

pub const T_NUM_INT: Dt = T_NUMBER | (0 << 4);
pub const T_NUM_FLT: Dt = T_NUMBER | (1 << 4);

pub const T_LCL: Dt = T_FUNCTION | (0 << 4);
pub const T_LRF: Dt = T_FUNCTION | (1 << 4);
pub const T_CCL: Dt = T_FUNCTION | (2 << 4);

pub const T_LNG_STR: Dt = T_STRING | (0 << 4);
pub const T_SHR_STR: Dt = T_STRING | (1 << 4);

pub type TObj = LuaTObject;

#[derive(Debug, Clone, Copy)]
pub struct ObjectType(Dt);

impl Default for ObjectType {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl ObjectType {
    #[inline(always)]
    pub fn is_function(&self) -> bool {
        if self.0 & T_FUNCTION == T_FUNCTION {
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn into_inner(&self) -> Dt {
        self.0
    }
}

#[repr(align(8))]
#[derive(Clone, Copy)]
pub struct LuaTObject {
    pub val: DataType,
    pub val_idx: ObjectType,
}

impl Default for LuaTObject {
    fn default() -> Self {
        Self {
            val: Default::default(),
            val_idx: Default::default(),
        }
    }
}

#[repr(align(8))]
#[derive(Clone, Copy)]
pub enum DataType {
    UserData(Option<*mut ()>),
    Function(Option<FFUNC>),
    Bool(Option<bool>),
    Integer(Option<INT>),
    Number(Option<FLT>),
    Nil(Option<()>),
}

impl Default for DataType {
    fn default() -> Self {
        Self::Nil(Some(()))
    }
}

pub trait ObjectTrait {
    fn new(self) -> LuaTObject;

    fn set_value(self, obj: &mut LuaTObject);

    fn into_inner(obj: &LuaTObject) -> Self;
}

impl ObjectTrait for ErrCode {
    fn new(self) -> LuaTObject {
        LuaTObject {
            val: DataType::Nil(None),
            val_idx: ObjectType(self.0),
        }
    }

    fn set_value(self, obj: &mut LuaTObject) {
        obj.val = DataType::Nil(None);
        obj.val_idx = ObjectType(self.0);
    }

    fn into_inner(obj: &LuaTObject) -> Self {
        if let DataType::Nil(_id) = obj.val {
            return ErrCode(NONE_OBJECT);
        } else {
            ErrCode(obj.val_idx.0)
        }
    }
}

impl ObjectTrait for Option<*mut ()> {
    fn new(self) -> LuaTObject {
        LuaTObject {
            val_idx: ObjectType(T_LIGHT_USER_DATA),
            val: DataType::UserData(self),
        }
    }

    fn set_value(self, obj: &mut LuaTObject) {
        obj.val = DataType::UserData(self);
        obj.val_idx.0 = T_LIGHT_USER_DATA;
    }

    fn into_inner(obj: &LuaTObject) -> Self {
        if obj.val_idx.0 != T_LIGHT_USER_DATA {
            return None;
        }

        if let DataType::UserData(mut val) = obj.val {
            val.take()
        } else {
            None
        }
    }
}

impl ObjectTrait for Option<FFUNC> {
    fn new(self) -> LuaTObject {
        LuaTObject {
            val_idx: ObjectType(T_LRF),
            val: DataType::Function(self),
        }
    }

    fn set_value(self, obj: &mut LuaTObject) {
        obj.val = DataType::Function(self);
        obj.val_idx.0 = T_LRF;
    }

    fn into_inner(obj: &LuaTObject) -> Self {
        if obj.val_idx.0 != T_LRF {
            return None;
        }

        if let DataType::Function(mut val) = obj.val {
            val.take()
        } else {
            None
        }
    }
}

impl ObjectTrait for Option<bool> {
    fn new(self) -> LuaTObject {
        LuaTObject {
            val_idx: ObjectType(T_BOOLEAN),
            val: DataType::Bool(self),
        }
    }

    fn set_value(self, obj: &mut LuaTObject) {
        obj.val = DataType::Bool(self);
        obj.val_idx.0 = T_BOOLEAN;
    }

    fn into_inner(obj: &LuaTObject) -> Self {
        if obj.val_idx.0 != T_BOOLEAN {
            return None;
        }

        if let DataType::Bool(mut val) = obj.val {
            val.take()
        } else {
            None
        }
    }
}

impl ObjectTrait for Option<INT> {
    fn new(self) -> LuaTObject {
        LuaTObject {
            val_idx: ObjectType(T_NUM_INT),
            val: DataType::Integer(self),
        }
    }

    fn set_value(self, obj: &mut LuaTObject) {
        obj.val = DataType::Integer(self);
        obj.val_idx.0 = T_NUM_INT;
    }

    fn into_inner(obj: &LuaTObject) -> Self {
        if obj.val_idx.0 != T_NUM_INT {
            return None;
        }

        if let DataType::Integer(mut val) = obj.val {
            val.take()
        } else {
            None
        }
    }
}

impl ObjectTrait for Option<FLT> {
    fn new(self) -> LuaTObject {
        LuaTObject {
            val_idx: ObjectType(T_NUM_FLT),
            val: DataType::Number(self),
        }
    }

    fn set_value(self, obj: &mut LuaTObject) {
        obj.val = DataType::Number(self);
        obj.val_idx.0 = T_NUM_FLT;
    }

    fn into_inner(obj: &LuaTObject) -> Self {
        if obj.val_idx.0 != T_NUM_FLT {
            return None;
        }

        if let DataType::Number(mut val) = obj.val {
            val.take()
        } else {
            None
        }
    }
}

impl ObjectTrait for Option<()> {
    fn new(self) -> LuaTObject {
        LuaTObject {
            val_idx: ObjectType(T_NIL),
            val: DataType::Nil(self),
        }
    }

    fn set_value(self, obj: &mut LuaTObject) {
        obj.val = DataType::Nil(self);
        obj.val_idx.0 = T_NIL;
    }

    fn into_inner(obj: &LuaTObject) -> Self {
        if obj.val_idx.0 != T_NIL {
            return None;
        }

        if let DataType::Nil(mut val) = obj.val {
            val.take()
        } else {
            None
        }
    }
}

impl DataType {
    pub fn is_none(&self) -> bool {
        match self {
            DataType::UserData(val) => val.is_none(),
            DataType::Bool(val) => val.is_none(),
            DataType::Integer(val) => val.is_none(),
            DataType::Number(val) => val.is_none(),
            DataType::Function(val) => val.is_none(),
            DataType::Nil(_) => true,
        }
    }

    pub fn is_some(&self) -> bool {
        match self {
            DataType::UserData(val) => val.is_some(),
            DataType::Bool(val) => val.is_some(),
            DataType::Integer(val) => val.is_some(),
            DataType::Number(val) => val.is_some(),
            DataType::Function(val) => val.is_some(),
            DataType::Nil(_) => false,
        }
    }
}

