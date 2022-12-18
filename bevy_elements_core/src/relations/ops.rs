use super::bind::*;
use bevy::prelude::*;

fn transform<
    S: BindableSource,
    T: TryFrom<S, Error = E> + BindableTarget,
    E: Into<TransformationError>,
>(
    incoming: &S,
    mut current: Prop<T>,
) -> TransformationResult {
    let new_value = T::try_from(incoming.clone()).map_err(E::into)?;
    if new_value != *current {
        *current = new_value;
    }
    Ok(())
}

// from!(entity, Component:property) >> to!(entity, Component:property | filter)
impl<R: Component, W: Component, S: BindableSource, T: BindableTarget>
    std::ops::Shr<ToComponent<W, S, T>> for FromComponent<R, S>
{
    type Output = ComponentToComponent<R, W, S, T>;
    fn shr(self, rhs: ToComponent<W, S, T>) -> Self::Output {
        self.bind_component(rhs)
    }
}
// to!(entity, Component:property | filter) << from!(entity, Component:property)
impl<R: Component, W: Component, S: BindableSource, T: BindableTarget>
    std::ops::Shl<FromComponent<R, S>> for ToComponent<W, S, T>
{
    type Output = ComponentToComponent<R, W, S, T>;
    fn shl(self, from: FromComponent<R, S>) -> Self::Output {
        self.bind_component(from)
    }
}
// from!(entity, Component:property | filter) >> to!(entity, Component | property)
impl<R: Component, W: Component, S: BindableSource, T: BindableTarget>
    std::ops::Shr<ToComponentWithoutTransformer<W, T>> for FromComponentWithTransformer<R, S, T>
{
    type Output = ComponentToComponent<R, W, S, T>;
    fn shr(self, rhs: ToComponentWithoutTransformer<W, T>) -> Self::Output {
        rhs.bind_component(self)
    }
}
// to!(entity, Component | property) << from!(entity, Component:property | filter)
impl<R: Component, W: Component, S: BindableSource, T: BindableTarget>
    std::ops::Shl<FromComponentWithTransformer<R, S, T>> for ToComponentWithoutTransformer<W, T>
{
    type Output = ComponentToComponent<R, W, S, T>;
    fn shl(self, from: FromComponentWithTransformer<R, S, T>) -> Self::Output {
        self.bind_component(from)
    }
}
// from!(entity, Component:property) >> to!(entity, Component:property)
impl<R, W, S, T, E> std::ops::Shr<ToComponentWithoutTransformer<W, T>> for FromComponent<R, S>
where
    E: Into<TransformationError>,
    R: Component,
    W: Component,
    S: BindableSource,
    T: BindableTarget + TryFrom<S, Error = E>,
{
    type Output = ComponentToComponent<R, W, S, T>;
    fn shr(self, to: ToComponentWithoutTransformer<W, T>) -> Self::Output {
        ToComponent {
            id: to.id,
            target: to.target,
            reader: to.reader,
            writer: to.writer,
            transformer: transform::<S, T, E>,
        }
        .bind_component(self)
    }
}
// impl<R, W, T, E> std::ops::Shr<ToComponentWithoutTransformer<W, T>> for FromComponent<R, String>
// where
//     E: Into<TransformationError>,
//     R: Component,
//     W: Component,
//     T: BindableTarget + std::str::FromStr<Err = E>,
// {
//     type Output = ComponentToComponent<R, W, String, T>;
//     fn shr(self, to: ToComponentWithoutTransformer<W, T>) -> Self::Output {
//         ToComponent {
//             id: to.id,
//             target: to.target,
//             reader: to.reader,
//             writer: to.writer,
//             transformer: |s, t| Ok(())
//         }
//         .bind_component(self)
//     }
// }
// to!(entity, Component:property) << from!(entity, Component:property)
impl<R, W, S, T, E> std::ops::Shl<FromComponent<R, S>> for ToComponentWithoutTransformer<W, T>
where
    E: Into<TransformationError>,
    R: Component,
    W: Component,
    S: BindableSource,
    T: BindableTarget + TryFrom<S, Error = E>,
{
    type Output = ComponentToComponent<R, W, S, T>;
    fn shl(self, from: FromComponent<R, S>) -> Self::Output {
        ToComponent {
            id: self.id,
            target: self.target,
            reader: self.reader,
            writer: self.writer,
            transformer: transform::<S, T, E>,
        }
        .bind_component(from)
    }
}
// from!(Resource:property) >> to!(entity, Component:property | filter)
impl<R: Resource, W: Component, S: BindableSource, T: BindableTarget>
    std::ops::Shr<ToComponent<W, S, T>> for FromResource<R, S>
{
    type Output = ResourceToComponent<R, W, S, T>;
    fn shr(self, rhs: ToComponent<W, S, T>) -> Self::Output {
        self.bind_component(rhs)
    }
}
// to!(entity, Component:property | filter) << from!(Resource:property)
impl<R: Resource, W: Component, S: BindableSource, T: BindableTarget>
    std::ops::Shl<FromResource<R, S>> for ToComponent<W, S, T>
{
    type Output = ResourceToComponent<R, W, S, T>;
    fn shl(self, rhs: FromResource<R, S>) -> Self::Output {
        self.bind_resource(rhs)
    }
}
// from!(Resource:property | filter) >> to!(entity, Component:property)
impl<R: Resource, W: Component, S: BindableSource, T: BindableTarget>
    std::ops::Shr<ToComponentWithoutTransformer<W, T>> for FromResourceWithTransformer<R, S, T>
{
    type Output = ResourceToComponent<R, W, S, T>;
    fn shr(self, rhs: ToComponentWithoutTransformer<W, T>) -> Self::Output {
        self.bind_component(rhs)
    }
}
// to!(entity, Component:property) << from!(Resource:property | filter)
impl<R: Resource, W: Component, S: BindableSource, T: BindableTarget>
    std::ops::Shl<FromResourceWithTransformer<R, S, T>> for ToComponentWithoutTransformer<W, T>
{
    type Output = ResourceToComponent<R, W, S, T>;
    fn shl(self, lhs: FromResourceWithTransformer<R, S, T>) -> Self::Output {
        self.bind_resource(lhs)
    }
}
// from!(Resource:property) >> to!(entity, Component:property)
impl<R, W, S, T, E> std::ops::Shr<ToComponentWithoutTransformer<W, T>> for FromResource<R, S>
where
    E: Into<TransformationError>,
    R: Resource,
    W: Component,
    S: BindableSource,
    T: BindableTarget + TryFrom<S, Error = E>,
{
    type Output = ResourceToComponent<R, W, S, T>;
    fn shr(self, to: ToComponentWithoutTransformer<W, T>) -> Self::Output {
        ToComponent {
            id: to.id,
            target: to.target,
            reader: to.reader,
            writer: to.writer,
            transformer: transform::<S, T, E>,
        }
        .bind_resource(self)
    }
}
// to!(entity, Component:property) << from!(entity, Component:property)
impl<R, W, S, T, E> std::ops::Shl<FromResource<R, S>> for ToComponentWithoutTransformer<W, T>
where
    E: Into<TransformationError>,
    R: Resource,
    W: Component,
    S: BindableSource,
    T: BindableTarget + TryFrom<S, Error = E>,
{
    type Output = ResourceToComponent<R, W, S, T>;
    fn shl(self, from: FromResource<R, S>) -> Self::Output {
        ToComponent {
            id: self.id,
            target: self.target,
            reader: self.reader,
            writer: self.writer,
            transformer: transform::<S, T, E>,
        }
        .bind_resource(from)
    }
}
