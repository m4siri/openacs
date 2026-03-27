use crate::error::Error;
use cwmp_xsd_types::XsiType;

#[derive(Debug)]
pub enum AttributeNotificationValue {
    #[doc = "Notification off. The CPE need not inform the ACS of a change to the specified parameter(s)"]
    _0,
    #[doc = "Passive notification. Whenever the specified parameter value changes, the CPE MUST include the new value in the ParameterList in the Inform message that is sent the next time a session is established to the ACS"]
    _1,
    #[doc = "Active notification. Whenever the specified parameter value changes, the CPE MUST initiate a session to the ACS, and include the new value in the ParameterList in the associated Inform message"]
    _2,
    #[doc = "Reserved for future use"]
    _3,
    #[doc = "Reserved for future use"]
    _4,
    #[doc = "Reserved for future use"]
    _5,
    #[doc = "Reserved for future use"]
    _6,
}

pub type SetParameterAttributesNotification = AttributeNotificationValue;

#[derive(Debug)]
pub struct ParameterList<T>(pub Vec<T>);

#[derive(Debug)]
pub struct AccessList(pub Vec<AccessListMember>);

#[derive(Debug)]
pub enum AccessListMember {
    Subscriber,
}

#[derive(Debug)]
pub struct SetParameterAttributesStruct {
    name: Option<String>,
    notification_change: bool,
    notification: SetParameterAttributesNotification,
    access_list_change: bool,
    // i have a feeling some vendors implement this wrong and not send the
    // attribute instead of doing soap-enc:arrayType="xsd:string[0]"
    access_list: AccessList,
}

#[derive(Debug)]
pub struct ParameterValueStruct {
    pub name: String,
    pub value: ParameterValueStructValue,
}

#[derive(Debug)]
pub enum ParameterValueType {
    Qualified {
        namespace: String,
        prefix: String,
        type_: String,
    },
    Unqualified(String),
}

impl ToString for ParameterValueType {
    fn to_string(&self) -> String {
        match self {
            ParameterValueType::Qualified {
                namespace,
                prefix,
                type_,
            } => format!("{}:{}", &prefix, &type_),
            ParameterValueType::Unqualified(inner) => inner.to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct ParameterValueStructValue {
    pub type_: Option<ParameterValueType>,
    pub value: String,
}

#[derive(Debug)]
pub struct GetParameterNames {
    parameter_path: ParameterName,
    next_level: bool,
}

#[derive(Debug)]
pub struct ParameterNames(pub Vec<ParameterName>);

#[derive(Debug)]
pub enum ParameterName {
    Path(String),
    Full(String),
    WildCard(String),
    WildCardPath(String),
    None, // refer: 3.6.2, A.2.4
}

impl TryFrom<String> for ParameterName {
    type Error = Error;
    fn try_from(input: String) -> Result<Self, Self::Error> {
        if input.ends_with('*') || input.ends_with("*.") {
            return Err(Error::InvalidParameterName(
                "Wildcard '*' cannot be the last part of a path.".to_string(),
            ));
        }

        let partial = input.ends_with('.');
        let wildcard = input.contains('*');

        let parameter_name = match (partial, wildcard) {
            (true, true) => Self::WildCardPath(input), // ends with '.' and contains '*'
            (true, false) => Self::Path(input),        // ends with '.' only
            (false, true) => Self::WildCard(input),    // contains '*' but doesn't end with '.'
            (false, false) => Self::Full(input),       // neither
        };

        Ok(parameter_name)
    }
}

impl ToString for ParameterName {
    fn to_string(&self) -> String {
        match self {
            Self::Path(inner) => inner,
            Self::Full(inner) => inner,
            Self::WildCard(inner) => inner,
            Self::WildCardPath(inner) => inner,
            Self::None => "",
        }
        .to_string()
    }
}

impl TryFrom<(String, cwmp_xsd_schema::ValueType)> for ParameterValueStruct {
    type Error = Error;
    fn try_from((name, value): (String, cwmp_xsd_schema::ValueType)) -> Result<Self, Self::Error> {
        let value = {
            ParameterValueStructValue {
                //XXX this actually handles the case where the value of `xsi:type="xsd:string"`,
                // not sure if any other values can exist i.e unqualified value.
                type_: value.type_.map(|attr_value| ParameterValueType::Qualified {
                    namespace: attr_value.namespace,
                    prefix: attr_value.prefix,
                    type_: attr_value.type_,
                }),
                value: value.content,
            }
        };
        Ok(Self { name, value })
    }
}

fn extract_array_content<T, I, F>(
    soap_array: cwmp_xsd_schema::soapenc::ArrayType,
    filter_expr: F,
) -> Result<T, Error>
where
    F: Fn(cwmp_xsd_schema::soapenc::ArrayTypeContent) -> Option<I>,
    T: From<Vec<I>>,
{
    let Some(array_type) = soap_array.array_type else {
        return Err(Error::MissingAttribute("arrayType".to_string()));
    };
    let content = soap_array.content;
    //TODO call stringify on the type and check if the nae matches
    let elem_length = array_type
        .trim()
        .split('[')
        .nth(1)
        .and_then(|part| part.split(']').next())
        .ok_or_else(|| Error::InvalidValue(format!("{}", array_type)))?
        .parse::<u32>()? as usize;

    let elements = content
        .into_iter()
        .filter_map(filter_expr)
        .collect::<Vec<I>>();

    if elements.len() > elem_length {
        return Err(Error::InvalidValue(format!(
            "Expected {} elements but recieved only {}.",
            elem_length,
            elements.len()
        )));
    }

    Ok(T::from(elements))
}

impl
    TryFrom<(
        xsd_parser_types::xml::Nillable<xsd_parser_types::xml::Nillable<String>>,
        bool,
    )> for GetParameterNames
{
    type Error = Error;
    fn try_from(
        (parameter_path, next_level): (
            xsd_parser_types::xml::Nillable<xsd_parser_types::xml::Nillable<String>>,
            bool,
        ),
    ) -> Result<Self, Self::Error> {
        (
            parameter_path.into_inner().and_then(|v| v.into_inner()),
            next_level,
        )
            .try_into()
    }
}

impl TryFrom<(Option<String>, bool)> for GetParameterNames {
    type Error = Error;
    fn try_from((parameter_path, next_level): (Option<String>, bool)) -> Result<Self, Self::Error> {
        let parameter_path: ParameterName = parameter_path
            .map(|v| v.try_into())
            .unwrap_or(Ok(ParameterName::None))?;

        Ok(Self {
            parameter_path,
            next_level,
        })
    }
}

impl TryFrom<Vec<String>> for ParameterNames {
    type Error = Error;

    fn try_from(input: Vec<String>) -> Result<Self, Self::Error> {
        let mut names: Vec<ParameterName> = Vec::new();

        for name in input.into_iter() {
            names.push(name.try_into()?);
        }

        Ok(ParameterNames(names))
    }
}

impl From<cwmp_xsd_schema::cwmp_12::ParameterAttributeNotificationValueType>
    for AttributeNotificationValue
{
    fn from(soap_array: cwmp_xsd_schema::cwmp_12::ParameterAttributeNotificationValueType) -> Self {
        use cwmp_xsd_schema::cwmp_12::ParameterAttributeNotificationValueType as N;
        match soap_array {
            N::_0 => AttributeNotificationValue::_0,
            N::_1 => AttributeNotificationValue::_1,
            N::_2 => AttributeNotificationValue::_2,
            N::_3 => AttributeNotificationValue::_3,
            N::_4 => AttributeNotificationValue::_4,
            N::_5 => AttributeNotificationValue::_5,
            N::_6 => AttributeNotificationValue::_6,
        }
    }
}

macro_rules! impl_try_from_array_type {
    ($ty: ty, $inner: expr ) => {
        impl TryFrom<cwmp_xsd_schema::soapenc::ArrayType> for $ty {
            type Error = Error;
            fn try_from(
                soap_array: cwmp_xsd_schema::soapenc::ArrayType,
            ) -> Result<Self, Self::Error> {
                let values = { $inner };
                extract_array_content(soap_array, values)
            }
        }
    };
}

impl_try_from_array_type! {
    ParameterList<ParameterValueStruct>,
    {
        use cwmp_xsd_schema::soapenc::ArrayTypeContent;

        let values = |content: ArrayTypeContent|  {

            let log_err = |e: &Error| eprintln!("Error converting to ParameterValueStruct: {e}");

            match content {
                ArrayTypeContent::ParameterValueStruct10(inner) => (inner.name, inner.value)
                    .try_into()
                    .inspect_err(log_err)
                    .ok(),
                ArrayTypeContent::ParameterValueStruct11(inner) => (inner.name, inner.value)
                    .try_into()
                    .inspect_err(log_err)
                    .ok(),
                ArrayTypeContent::ParameterValueStruct12(inner) => (inner.name, inner.value)
                    .try_into()
                    .inspect_err(log_err)
                    .ok(),

                _ => None,
            }
        };

        values

    }
}

impl_try_from_array_type! {
        AccessList,
        {
            use cwmp_xsd_schema::soapenc::ArrayTypeContent;

            let values = |content: ArrayTypeContent| match content {
                ArrayTypeContent::String(inner) => match inner.as_str() {
                    "Subscriber" => Some(AccessListMember::Subscriber),
                    _ => None,
                },
                _ => None,
            };
            values
        }

}

impl_try_from_array_type! {
        ParameterNames,
        {
           use cwmp_xsd_schema::soapenc::ArrayTypeContent;

           let values = |content: ArrayTypeContent| match content {
               ArrayTypeContent::String(inner) => inner
                   .try_into()
                   .inspect_err(|e|eprintln!("Error converting to ParameterNames: {e}"))
                   .ok(),
               _ => None,
           };
           values
        }

}

impl_try_from_array_type! {
    ParameterList<SetParameterAttributesStruct>,
    {
        use cwmp_xsd_schema::soapenc::ArrayTypeContent;

        let values: _ = |content: ArrayTypeContent| {
            let log_err = |e: &Error| eprintln!("Error converting to AccessList: {e}");
            match content {
                ArrayTypeContent::SetParameterAttributesStruct10(inner) => {
                    let access_list: AccessList =
                        inner.access_list.try_into().inspect_err(log_err).ok()?;
                    let notification: SetParameterAttributesNotification =
                        inner.notification.into();
                    let name = inner.name.into_inner().and_then(|v| v.into_inner());
                    Some(SetParameterAttributesStruct {
                        name,
                        access_list,
                        notification,
                        access_list_change: inner.access_list_change,
                        notification_change: inner.notification_change,
                    })
                }
                ArrayTypeContent::SetParameterAttributesStruct11(inner) => {
                    let access_list: AccessList =
                        (*inner.access_list).try_into().inspect_err(log_err).ok()?;
                    let notification: SetParameterAttributesNotification =
                        inner.notification.into();
                    let name = inner.name.into_inner().and_then(|v| v.into_inner());
                    Some(SetParameterAttributesStruct {
                        name,
                        access_list,
                        notification,
                        access_list_change: inner.access_list_change,
                        notification_change: inner.notification_change,
                    })
                }
                ArrayTypeContent::SetParameterAttributesStruct12(inner) => {
                    let access_list: AccessList =
                        (*inner.access_list).try_into().inspect_err(log_err).ok()?;
                    let notification: SetParameterAttributesNotification =
                        inner.notification.into();
                    let name = Some(inner.name);
                    Some(SetParameterAttributesStruct {
                        name,
                        access_list,
                        notification,
                        access_list_change: inner.access_list_change,
                        notification_change: inner.notification_change,
                    })
                }
                _ => None,
            }
        };
        values
    }
}

macro_rules! impl_from_vec {
    ($list_ty: ty, $item: ty) => {
        impl From<Vec<$item>> for $list_ty {
            fn from(input: Vec<$item>) -> Self {
                Self(input)
            }
        }
    };
}

impl_from_vec!(AccessList, AccessListMember);
impl_from_vec!(ParameterNames, ParameterName);
impl_from_vec!(ParameterList<ParameterValueStruct>, ParameterValueStruct);
impl_from_vec!(
    ParameterList<SetParameterAttributesStruct>,
    SetParameterAttributesStruct
);

macro_rules! impl_from_set_parameter_attributes_struct_notification {
    ($($ty: ty),+ ) => {
        $(
            impl From<$ty> for AttributeNotificationValue {
                fn from(input: $ty) -> Self {
                    match input {
                        <$ty>::_0 => AttributeNotificationValue::_0,
                        <$ty>::_1 => AttributeNotificationValue::_1,
                        <$ty>::_2 => AttributeNotificationValue::_2,
                    }
                }
            }
        )+
    };
}

impl_from_set_parameter_attributes_struct_notification!(
    cwmp_xsd_schema::SetParameterAttributesStructNotificationElementType,
    cwmp_xsd_schema::tns::SetParameterAttributesStructNotificationElementType
);
