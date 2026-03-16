// RHYM uses the QEM format under the hood for storing map data.
//
// The only difference to the format is having required parameters and
// having non-rhythia object data stripped.
//
// To indicate this, the file extension should be .rhym instead of .qem and the signature block
// would be `RHYM` instead of `IQEM`.
//
// To be documented.

pub mod serde;
