use std::collections::HashMap;

use inkwell::types::{BasicTypeEnum};
use inkwell::values::{FunctionValue, PointerValue};
use crate::ast::CompilationUnit;

#[cfg(test)]
mod tests;
mod visitor;


#[derive(Debug)]
pub struct VariableIndexEntry<'ctx>{
    name                    : String,
    information             : VariableInformation,
    generated_reference     : Option<PointerValue<'ctx>>,
}

#[derive(Debug)]
pub struct DataTypeIndexEntry<'ctx> {
    name                    : String,
    implementation    : Option<FunctionValue<'ctx>>,    // the generated function to all if this type is callable
    generated_type          : Option<BasicTypeEnum<'ctx>>,    //the datatype (struct, enum, etc.)
}

impl <'ctx> VariableIndexEntry<'ctx> {
    pub fn associate(&mut self, generated_reference: PointerValue<'ctx>) {
        self.generated_reference = Some(generated_reference);
    }

    pub fn get_type_name(&self) -> &str {
        self.information.data_type_name.as_str()
    }

    pub fn get_generated_reference(&self) -> Option<PointerValue<'ctx>> {
        self.generated_reference
    }
}
impl <'ctx> DataTypeIndexEntry<'ctx> {
    pub fn associate_type(&mut self, generated_type: BasicTypeEnum<'ctx>) {
        self.generated_type = Some(generated_type);
    }

    pub fn associate_implementation(&mut self, implementation: FunctionValue<'ctx>) {
        self.implementation = Some(implementation);
    }

    pub fn get_type(&self) -> Option<BasicTypeEnum<'ctx>> {
        self.generated_type
    }

    pub fn get_implementation(&self) -> Option<FunctionValue<'ctx>> {
        self.implementation
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VariableType { Local, Input, Output, InOut, Global, Return }

/// information regarding a variable
#[derive(Debug)]
pub struct VariableInformation {
    /// the type of variable
    variable_type   : VariableType, 
    /// the variable's datatype
    data_type_name  : String,
    /// the variable's qualifier
    qualifier       : Option<String>, 
}

#[derive(Debug)]
pub enum DataTypeType { 
    Scalar,      // built in types: INT, BOOL, WORD, ... 
    Struct,         // Struct-DataType
    FunctionBlock,  // a Functionblock instance
    AliasType       // a Custom-Alias-dataType 
}

/// information regarding a custom datatype
#[derive(Debug)]
pub struct DataTypeInformation {
    /// what kind of datatype is this
    kind        : DataTypeType,
}


/// The global index of the rusty-compiler
/// 
/// The index contains information about all referencable elements. Furthermore it
/// contains information about the type-system of the compiled program.
/// 
/// TODO: consider String-references
/// 
pub struct Index<'ctx> {
    /// all global variables
    global_variables    : HashMap<String, VariableIndexEntry<'ctx>>,

    /// all local variables, grouped by the POU's name
    local_variables     : HashMap<String, HashMap<String, VariableIndexEntry<'ctx>>>,

    /// all types (structs, enums, type, POUs, etc.)
    types               : HashMap<String, DataTypeIndexEntry<'ctx>>,
}

impl<'ctx> Index<'ctx> {
    pub fn new() -> Index<'ctx> {
        let mut index = Index {
            global_variables : HashMap::new(),
            local_variables : HashMap::new(),
            types : HashMap::new(),   
        };

        index.types.insert("Int".to_string(), DataTypeIndexEntry{
            name: "Int".to_string(),
            generated_type: None,
            implementation: None,
        });
        index.types.insert("Bool".to_string(), DataTypeIndexEntry{
            name: "Bool".to_string(),
            generated_type: None,
            implementation: None,
        });
        index
    }

    pub fn find_global_variable(&self, name: &str) -> Option<&VariableIndexEntry<'ctx>> {
        self.global_variables.get(name)
    }

    pub fn find_member(&self, pou_name: &str, variable_name: &str) -> Option<&VariableIndexEntry<'ctx>>{
        self.local_variables.get(pou_name)
            .and_then(|map| map.get(variable_name))
    }

    pub fn find_variable(&self, context : Option<&str>, variable_name  : &str)  -> Option<&VariableIndexEntry<'ctx>> {
        match context {
            Some(context) => self.find_member(context, variable_name).or_else(||self.find_global_variable(variable_name)),
            None => self.find_global_variable(variable_name)
        }
    }

    pub fn find_type(&self, type_name : &str) -> Option<&DataTypeIndexEntry<'ctx>> {
        self.types.get(type_name)
    }

    pub fn find_callable_instance_variable(&self, context: Option<&str>, reference : &str) -> Option<&VariableIndexEntry<'ctx>> {
        //look for a *callable* variable with that name
        self.find_variable(context, reference).filter(|v|
            {
                //callable means, there is an implementation associated with the variable's datatype
                self.find_type(v.information.data_type_name.as_str()).map(|it| it.implementation).flatten().is_some()
            }
        )
    }

    pub fn register_local_variable(&mut self, 
                                        pou_name: String, 
                                        variable_name: String, 
                                        variable_type: VariableType, 
                                        type_name: String) {
        
        let locals = self.local_variables.entry(pou_name.clone()).or_insert_with(|| HashMap::new());

        let entry = VariableIndexEntry{
            name : variable_name.clone(),
            information : VariableInformation {
                variable_type: variable_type,
                data_type_name: type_name,
                qualifier: Some(pou_name.clone()),
            },
            generated_reference: None,
        };                         
        locals.insert(variable_name, entry);
    }

    pub fn associate_local_variable(&mut self,
        pou_name : &str,
        variable_name: &str,
        value: PointerValue<'ctx>){
            if let Some(entry) = self.local_variables.get_mut(pou_name) 
                .and_then(|map| map.get_mut(variable_name)) {
                    entry.generated_reference = Some(value);
                }
    }

    pub fn register_global_variable(&mut self,
                                name: String, 
                                type_name: String){
        
        let entry = VariableIndexEntry{
            name : name.clone(),
            information : VariableInformation {
                variable_type: VariableType::Global,
                data_type_name: type_name, 
                qualifier: None,
            },
            generated_reference: None,
        };                         
        self.global_variables.insert(name, entry);

    }

    pub fn associate_global_variable(&mut self, name: &str, value: PointerValue<'ctx>) {
        if let Some(entry) = self.global_variables.get_mut(name) {
            entry.generated_reference = Some(value);
        }
    }

    pub fn associate_callable_implementation(&mut self, name : &str, value : FunctionValue<'ctx> ) {
        if let Some(entry) = self.types.get_mut(name) {
            entry.implementation = Some(value);
        };
    }

    pub fn associate_type(&mut self, name : &str, value : BasicTypeEnum<'ctx>) {
        if let Some(entry) = self.types.get_mut(name) {
            entry.generated_type = Some(value);
        };
    }

    pub fn print_global_variables(&self) {
        println!("{:?}", self.global_variables);
    }

    pub fn register_type(&mut self,
                        type_name: String) {

        let index_entry = DataTypeIndexEntry{
            name: type_name.clone(),
            generated_type: None,
            implementation : None,
        };
        self.types.insert(type_name, index_entry);
    }

    pub fn visit(&mut self, unit: &CompilationUnit) {
        visitor::visit(self, unit);
    }

}