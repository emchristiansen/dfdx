use crate::{
    shapes::*,
    tensor::{unique_id, Cpu, Tensor},
};

use std::vec::Vec;
impl<E: Dtype> super::UnstackKernel<E> for Cpu {
    fn forward<S: Shape>(
        &self,
        inp: &Tensor<S, E, Self>,
    ) -> Result<Vec<Tensor<S::Smaller, E, Self>>, Self::Err>
    where
        S: super::SubDim,
    {
        let shape: S::Smaller = inp.shape().sub_dim();
        let mut strides = shape.strides();
        for i in 0..S::Smaller::NUM_DIMS {
            strides[i] = inp.strides[i + 1];
        }

        let num_items = inp.shape().concrete()[0];

        let mut tensors = Vec::with_capacity(num_items);
        for i in 0..num_items {
            let item_size = inp.data.len() / num_items;

            let mut data = self.try_alloc_elem(item_size, E::default())?;
            data.copy_from_slice(&inp.data[i * item_size..(i + 1) * item_size]);

            tensors.push(Tensor {
                id: unique_id(),
                data: std::sync::Arc::new(data),
                shape,
                strides,
                device: self.clone(),
                tape: Default::default(),
            });
        }
        Ok(tensors)
    }

    fn backward(
        &self,
        grad_inp: &mut Self::Vec,
        grad_out: Vec<&Self::Vec>,
    ) -> Result<(), Self::Err> {
        let grad_out_len = grad_out.len();
        for (i, item) in grad_out.into_iter().enumerate() {
            for (j, value) in item.iter().enumerate() {
                let item_size = grad_inp.len() / grad_out_len;
                grad_inp[i * item_size + j] += *value;
            }
        }

        Ok(())
    }
}