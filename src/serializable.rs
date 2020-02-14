pub trait Serializable {
    fn serialized_size(&self) -> usize;
    fn serialize(&self, out: &mut [u8]) -> usize;
    fn serialize_to_box(&self) -> Box<[u8]> {
        let mut data = vec![0; self.serialized_size()].into_boxed_slice();
        self.serialize(&mut data);
        data
    }
}
