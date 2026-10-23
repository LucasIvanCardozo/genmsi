//! What the app says about an entity the model uses without declaring it.
//!
//! The rule comes from the original: the `StorageUse` class says "El Storage
//! debe estar declarado (esta clase no lo declara)", and the editor's entity
//! picker only offers the entities the model declares. A model that uses a
//! storage nobody declares is not wrong enough to refuse (the original opens
//! it), but it must not pass silently.

use genmsi_app_engine::session::diagnostics_of;
use genmsi_format::parse_str;

/// A model with a `StorageUse` that uses the storage `Almacen`, and an
/// optional `StorageDef` that declares it.
fn model(with_definition: bool) -> genmsi_format::GmdlModel {
    let definition = if with_definition {
        r#"<module moduleid="2" moduleclass="StorageDef" owner="0" caption="c" name="n" kind="A">
             <props><property name="Storage" hint="" value="Almacen" type="EntityDef" subType="Storage" visible="1" unique="1"/></props>
           </module>"#
    } else {
        ""
    };
    let xml = format!(
        r#"<gmodel versioncode="1"><gclasses/><modules>
             <module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C">
               <links/><inputs/><outputs/>
             </module>
             <module moduleid="1" moduleclass="StorageUse" owner="0" caption="c" name="n" kind="A">
               <inputs><input ioid="1" name="Entrada1" caption="" mylabel="L1"/></inputs>
               <outputs><output ioid="2" name="Salida1" caption="" mylabel="O1"/></outputs>
               <props>
                 <property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/>
                 <property name="Modificador" hint="" value="5" type="TSNA" subType="SNA" visible="1" unique="0"/>
                 <property name="Storage" hint="" value="Almacen" type="EntityUse" subType="Storage" visible="1" unique="0"/>
               </props>
             </module>
             {definition}
             <module moduleid="3" moduleclass="Terminate" owner="0" caption="c" name="n" kind="A">
               <inputs><input ioid="1" name="Entrada1" caption="" mylabel="L3"/></inputs>
               <props><property name="Unidades" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props>
             </module>
           </modules></gmodel>"#
    );
    parse_str(&xml).expect("the test model parses")
}

#[test]
fn the_app_shows_the_storage_the_model_does_not_declare() {
    let shown = diagnostics_of(&model(false));
    assert_eq!(shown.status, "diagnostics", "the app stays silent");
    let message = shown
        .diagnostics
        .iter()
        .find(|message| message.contains("Almacen"))
        .expect("the warning names the storage");
    assert!(message.contains("StorageDef"), "{message}");
}

#[test]
fn the_app_is_quiet_when_the_storage_is_declared() {
    let shown = diagnostics_of(&model(true));
    assert_eq!(shown.status, "ok", "{:?}", shown.diagnostics);
}
