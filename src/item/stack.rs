use serde::{Serialize, Deserialize};

/// An instance of an item with quantity
///
/// This represents a specific amount of an item type. It's stored
/// in inventory slots and can be split/merged with other stacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStack {
    /// ID of the item definition in ItemRegistry
    pub item_id: String,

    /// How many of this item (1 to max_stack_size)
    pub quantity: u32,
}

impl ItemStack {
    /// Creates a new item stack
    pub fn new(item_id: impl Into<String>, quantity: u32) -> Self {
        ItemStack {
            item_id: item_id.into(),
            quantity,
        }
    }

    /// Returns true if this stack can merge with another
    ///
    /// Stacks can merge if they're the same item type
    #[allow(dead_code)]  // Reserved for future stack merging feature
    pub fn can_merge_with(&self, other: &ItemStack) -> bool {
        self.item_id == other.item_id
    }

    /// Merges another stack into this one
    ///
    /// Returns how many items couldn't fit (overflow)
    ///
    /// # Example
    /// ```
    /// let mut stack1 = ItemStack::new("slime_ball", 50);
    /// let stack2 = ItemStack::new("slime_ball", 20);
    /// let overflow = stack1.merge(stack2, 64);
    /// assert_eq!(stack1.quantity, 64);
    /// assert_eq!(overflow, 6);
    /// ```
    #[allow(dead_code)]  // Reserved for future stack merging feature
    pub fn merge(&mut self, other: ItemStack, max_stack_size: u32) -> u32 {
        if !self.can_merge_with(&other) {
            return other.quantity;  // Can't merge, return all as overflow
        }

        let total = self.quantity + other.quantity;

        if total <= max_stack_size {
            // All items fit in this stack
            self.quantity = total;
            0  // No overflow
        } else {
            // Stack is full, some items overflow
            self.quantity = max_stack_size;
            total - max_stack_size  // Return overflow amount
        }
    }

    /// Splits this stack into two
    ///
    /// Takes `amount` items from this stack and returns them as a new stack.
    /// Returns None if there aren't enough items to split.
    ///
    /// # Example
    /// ```
    /// let mut stack = ItemStack::new("slime_ball", 10);
    /// let split = stack.split(3).unwrap();
    /// assert_eq!(stack.quantity, 7);
    /// assert_eq!(split.quantity, 3);
    /// ```
    #[allow(dead_code)]  // Reserved for future stack splitting feature (shift-click)
    pub fn split(&mut self, amount: u32) -> Option<ItemStack> {
        if amount == 0 || amount >= self.quantity {
            return None;  // Can't split 0 or entire stack
        }

        self.quantity -= amount;

        Some(ItemStack {
            item_id: self.item_id.clone(),
            quantity: amount,
        })
    }

    /// Takes up to `amount` items from this stack
    ///
    /// Returns how many items were actually taken (might be less if stack is small)
    ///
    /// # Example
    /// ```
    /// let mut stack = ItemStack::new("slime_ball", 5);
    /// let taken = stack.take(10);
    /// assert_eq!(taken, 5);  // Only had 5 to take
    /// assert_eq!(stack.quantity, 0);
    /// ```
    #[allow(dead_code)]  // Reserved for future partial stack taking
    pub fn take(&mut self, amount: u32) -> u32 {
        let taken = amount.min(self.quantity);
        self.quantity -= taken;
        taken
    }

    /// Adds items to this stack
    ///
    /// Returns how many items couldn't fit (overflow)
    pub fn add(&mut self, amount: u32, max_stack_size: u32) -> u32 {
        let total = self.quantity + amount;

        if total <= max_stack_size {
            self.quantity = total;
            0
        } else {
            self.quantity = max_stack_size;
            total - max_stack_size
        }
    }

    /// Returns true if this stack is empty
    #[allow(dead_code)]  // Reserved for future empty stack checks
    pub fn is_empty(&self) -> bool {
        self.quantity == 0
    }

    /// Splits this stack in half, returning a new stack with half the quantity.
    /// The original stack retains the other half (and any remainder).
    /// Returns None if the stack has 1 or fewer items.
    pub fn split_half(&mut self) -> Option<ItemStack> {
        if self.quantity <= 1 {
            return None; // Cannot split a stack of 1 or less
        }

        let new_stack_quantity = self.quantity / 2;
        self.quantity -= new_stack_quantity;

        Some(ItemStack::new(self.item_id.clone(), new_stack_quantity))
    }
}
