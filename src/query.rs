// This file is generated by rust-protobuf 2.18.2. Do not edit
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `query.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_18_2;

#[derive(PartialEq,Clone,Default)]
pub struct RangedRequest {
    // message fields
    pub symbol: ::std::string::String,
    pub series: ::std::string::String,
    pub first: ::protobuf::SingularPtrField<::protobuf::well_known_types::Timestamp>,
    pub last: ::protobuf::SingularPtrField<::protobuf::well_known_types::Timestamp>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a RangedRequest {
    fn default() -> &'a RangedRequest {
        <RangedRequest as ::protobuf::Message>::default_instance()
    }
}

impl RangedRequest {
    pub fn new() -> RangedRequest {
        ::std::default::Default::default()
    }

    // string symbol = 1;


    pub fn get_symbol(&self) -> &str {
        &self.symbol
    }
    pub fn clear_symbol(&mut self) {
        self.symbol.clear();
    }

    // Param is passed by value, moved
    pub fn set_symbol(&mut self, v: ::std::string::String) {
        self.symbol = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_symbol(&mut self) -> &mut ::std::string::String {
        &mut self.symbol
    }

    // Take field
    pub fn take_symbol(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.symbol, ::std::string::String::new())
    }

    // string series = 2;


    pub fn get_series(&self) -> &str {
        &self.series
    }
    pub fn clear_series(&mut self) {
        self.series.clear();
    }

    // Param is passed by value, moved
    pub fn set_series(&mut self, v: ::std::string::String) {
        self.series = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_series(&mut self) -> &mut ::std::string::String {
        &mut self.series
    }

    // Take field
    pub fn take_series(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.series, ::std::string::String::new())
    }

    // .google.protobuf.Timestamp first = 3;


    pub fn get_first(&self) -> &::protobuf::well_known_types::Timestamp {
        self.first.as_ref().unwrap_or_else(|| <::protobuf::well_known_types::Timestamp as ::protobuf::Message>::default_instance())
    }
    pub fn clear_first(&mut self) {
        self.first.clear();
    }

    pub fn has_first(&self) -> bool {
        self.first.is_some()
    }

    // Param is passed by value, moved
    pub fn set_first(&mut self, v: ::protobuf::well_known_types::Timestamp) {
        self.first = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_first(&mut self) -> &mut ::protobuf::well_known_types::Timestamp {
        if self.first.is_none() {
            self.first.set_default();
        }
        self.first.as_mut().unwrap()
    }

    // Take field
    pub fn take_first(&mut self) -> ::protobuf::well_known_types::Timestamp {
        self.first.take().unwrap_or_else(|| ::protobuf::well_known_types::Timestamp::new())
    }

    // .google.protobuf.Timestamp last = 4;


    pub fn get_last(&self) -> &::protobuf::well_known_types::Timestamp {
        self.last.as_ref().unwrap_or_else(|| <::protobuf::well_known_types::Timestamp as ::protobuf::Message>::default_instance())
    }
    pub fn clear_last(&mut self) {
        self.last.clear();
    }

    pub fn has_last(&self) -> bool {
        self.last.is_some()
    }

    // Param is passed by value, moved
    pub fn set_last(&mut self, v: ::protobuf::well_known_types::Timestamp) {
        self.last = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_last(&mut self) -> &mut ::protobuf::well_known_types::Timestamp {
        if self.last.is_none() {
            self.last.set_default();
        }
        self.last.as_mut().unwrap()
    }

    // Take field
    pub fn take_last(&mut self) -> ::protobuf::well_known_types::Timestamp {
        self.last.take().unwrap_or_else(|| ::protobuf::well_known_types::Timestamp::new())
    }
}

impl ::protobuf::Message for RangedRequest {
    fn is_initialized(&self) -> bool {
        for v in &self.first {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.last {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.symbol)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.series)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.first)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.last)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.symbol.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.symbol);
        }
        if !self.series.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.series);
        }
        if let Some(ref v) = self.first.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.last.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if !self.symbol.is_empty() {
            os.write_string(1, &self.symbol)?;
        }
        if !self.series.is_empty() {
            os.write_string(2, &self.series)?;
        }
        if let Some(ref v) = self.first.as_ref() {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.last.as_ref() {
            os.write_tag(4, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> RangedRequest {
        RangedRequest::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "symbol",
                |m: &RangedRequest| { &m.symbol },
                |m: &mut RangedRequest| { &mut m.symbol },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "series",
                |m: &RangedRequest| { &m.series },
                |m: &mut RangedRequest| { &mut m.series },
            ));
            fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<::protobuf::well_known_types::Timestamp>>(
                "first",
                |m: &RangedRequest| { &m.first },
                |m: &mut RangedRequest| { &mut m.first },
            ));
            fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<::protobuf::well_known_types::Timestamp>>(
                "last",
                |m: &RangedRequest| { &m.last },
                |m: &mut RangedRequest| { &mut m.last },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<RangedRequest>(
                "RangedRequest",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static RangedRequest {
        static instance: ::protobuf::rt::LazyV2<RangedRequest> = ::protobuf::rt::LazyV2::INIT;
        instance.get(RangedRequest::new)
    }
}

impl ::protobuf::Clear for RangedRequest {
    fn clear(&mut self) {
        self.symbol.clear();
        self.series.clear();
        self.first.clear();
        self.last.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for RangedRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for RangedRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct DataPoint {
    // message fields
    pub timestamp: ::protobuf::SingularPtrField<::protobuf::well_known_types::Timestamp>,
    pub value: f64,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a DataPoint {
    fn default() -> &'a DataPoint {
        <DataPoint as ::protobuf::Message>::default_instance()
    }
}

impl DataPoint {
    pub fn new() -> DataPoint {
        ::std::default::Default::default()
    }

    // .google.protobuf.Timestamp timestamp = 1;


    pub fn get_timestamp(&self) -> &::protobuf::well_known_types::Timestamp {
        self.timestamp.as_ref().unwrap_or_else(|| <::protobuf::well_known_types::Timestamp as ::protobuf::Message>::default_instance())
    }
    pub fn clear_timestamp(&mut self) {
        self.timestamp.clear();
    }

    pub fn has_timestamp(&self) -> bool {
        self.timestamp.is_some()
    }

    // Param is passed by value, moved
    pub fn set_timestamp(&mut self, v: ::protobuf::well_known_types::Timestamp) {
        self.timestamp = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_timestamp(&mut self) -> &mut ::protobuf::well_known_types::Timestamp {
        if self.timestamp.is_none() {
            self.timestamp.set_default();
        }
        self.timestamp.as_mut().unwrap()
    }

    // Take field
    pub fn take_timestamp(&mut self) -> ::protobuf::well_known_types::Timestamp {
        self.timestamp.take().unwrap_or_else(|| ::protobuf::well_known_types::Timestamp::new())
    }

    // double value = 2;


    pub fn get_value(&self) -> f64 {
        self.value
    }
    pub fn clear_value(&mut self) {
        self.value = 0.;
    }

    // Param is passed by value, moved
    pub fn set_value(&mut self, v: f64) {
        self.value = v;
    }
}

impl ::protobuf::Message for DataPoint {
    fn is_initialized(&self) -> bool {
        for v in &self.timestamp {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.timestamp)?;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed64 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_double()?;
                    self.value = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.timestamp.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if self.value != 0. {
            my_size += 9;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.timestamp.as_ref() {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if self.value != 0. {
            os.write_double(2, self.value)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> DataPoint {
        DataPoint::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<::protobuf::well_known_types::Timestamp>>(
                "timestamp",
                |m: &DataPoint| { &m.timestamp },
                |m: &mut DataPoint| { &mut m.timestamp },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeDouble>(
                "value",
                |m: &DataPoint| { &m.value },
                |m: &mut DataPoint| { &mut m.value },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<DataPoint>(
                "DataPoint",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static DataPoint {
        static instance: ::protobuf::rt::LazyV2<DataPoint> = ::protobuf::rt::LazyV2::INIT;
        instance.get(DataPoint::new)
    }
}

impl ::protobuf::Clear for DataPoint {
    fn clear(&mut self) {
        self.timestamp.clear();
        self.value = 0.;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DataPoint {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DataPoint {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct TimeSeries {
    // message fields
    pub data: ::protobuf::RepeatedField<DataPoint>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a TimeSeries {
    fn default() -> &'a TimeSeries {
        <TimeSeries as ::protobuf::Message>::default_instance()
    }
}

impl TimeSeries {
    pub fn new() -> TimeSeries {
        ::std::default::Default::default()
    }

    // repeated .query.DataPoint data = 1;


    pub fn get_data(&self) -> &[DataPoint] {
        &self.data
    }
    pub fn clear_data(&mut self) {
        self.data.clear();
    }

    // Param is passed by value, moved
    pub fn set_data(&mut self, v: ::protobuf::RepeatedField<DataPoint>) {
        self.data = v;
    }

    // Mutable pointer to the field.
    pub fn mut_data(&mut self) -> &mut ::protobuf::RepeatedField<DataPoint> {
        &mut self.data
    }

    // Take field
    pub fn take_data(&mut self) -> ::protobuf::RepeatedField<DataPoint> {
        ::std::mem::replace(&mut self.data, ::protobuf::RepeatedField::new())
    }
}

impl ::protobuf::Message for TimeSeries {
    fn is_initialized(&self) -> bool {
        for v in &self.data {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.data)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.data {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        for v in &self.data {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> TimeSeries {
        TimeSeries::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<DataPoint>>(
                "data",
                |m: &TimeSeries| { &m.data },
                |m: &mut TimeSeries| { &mut m.data },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<TimeSeries>(
                "TimeSeries",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static TimeSeries {
        static instance: ::protobuf::rt::LazyV2<TimeSeries> = ::protobuf::rt::LazyV2::INIT;
        instance.get(TimeSeries::new)
    }
}

impl ::protobuf::Clear for TimeSeries {
    fn clear(&mut self) {
        self.data.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for TimeSeries {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for TimeSeries {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0bquery.proto\x12\x05query\x1a\x1fgoogle/protobuf/timestamp.proto\"\
    \xa1\x01\n\rRangedRequest\x12\x16\n\x06symbol\x18\x01\x20\x01(\tR\x06sym\
    bol\x12\x16\n\x06series\x18\x02\x20\x01(\tR\x06series\x120\n\x05first\
    \x18\x03\x20\x01(\x0b2\x1a.google.protobuf.TimestampR\x05first\x12.\n\
    \x04last\x18\x04\x20\x01(\x0b2\x1a.google.protobuf.TimestampR\x04last\"[\
    \n\tDataPoint\x128\n\ttimestamp\x18\x01\x20\x01(\x0b2\x1a.google.protobu\
    f.TimestampR\ttimestamp\x12\x14\n\x05value\x18\x02\x20\x01(\x01R\x05valu\
    e\"2\n\nTimeSeries\x12$\n\x04data\x18\x01\x20\x03(\x0b2\x10.query.DataPo\
    intR\x04data2{\n\nMarketData\x122\n\x05Query\x12\x14.query.RangedRequest\
    \x1a\x11.query.TimeSeries\"\0\x129\n\x0bQueryStream\x12\x14.query.Ranged\
    Request\x1a\x10.query.DataPoint\"\00\x01B#Z!github.com/luckless-finance/\
    queryb\x06proto3\
";

static file_descriptor_proto_lazy: ::protobuf::rt::LazyV2<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::LazyV2::INIT;

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    file_descriptor_proto_lazy.get(|| {
        parse_descriptor_proto()
    })
}
